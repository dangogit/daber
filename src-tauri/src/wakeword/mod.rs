//! Always-on wake phrase spotting ("hey Claude").
//!
//! Handy normally starts a recording from a global shortcut. This module adds a
//! second trigger: a lightweight classifier listens to the idle microphone
//! stream and fires the *same* action when it hears the wake phrase, so the
//! whole downstream pipeline (recording, VAD, transcription, paste) is reused
//! unchanged.
//!
//! Audio arrives from the recorder's monitor callback, which only fires while
//! Handy is NOT recording — that is what keeps the spotter from hearing (and
//! re-triggering on) the user's own dictation.
//!
//! The detector sits behind [`WakewordDetector`] so the backend can be swapped
//! without touching the runner, and all the decision logic lives in
//! [`SpotterCore`], which is a plain synchronous struct — the thread around it
//! is only plumbing.

mod livekit;
mod livekit_wakeword;

pub use livekit::LiveKitDetector;

use log::{debug, error, info, warn};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender, TrySendError};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Scores a window of 16 kHz mono audio for the wake phrase.
pub trait WakewordDetector: Send {
    /// Confidence in `0.0..=1.0` that the window contains the wake phrase.
    fn score(&mut self, window: &[i16]) -> anyhow::Result<f32>;
}

/// 2 s at 16 kHz — the window length the classifier is trained on. Shorter
/// windows produce too few embeddings and score zero.
pub const WINDOW_SAMPLES: usize = 32_000;

/// Fresh audio required between scoring passes (~250 ms). Overlapping windows
/// mean a phrase straddling a window boundary is still caught by the next pass.
pub const HOP_SAMPLES: usize = 4_000;

/// A pause longer than this means the buffer spans a recording (or a device
/// change), so its contents no longer form a continuous window. Cheaper and
/// more robust than coordinating with the recorder's state machine.
const CONTINUITY_GAP: Duration = Duration::from_millis(300);

/// Pending frames held before dropping. At the recorder's ~30 ms frames this is
/// nearly a second of slack; the audio thread must never block on the spotter,
/// and stale frames are worthless to a wake word anyway.
const QUEUE_DEPTH: usize = 32;

/// Sliding-window scoring with continuity handling.
///
/// A detection clears the window, so the next one cannot happen until a further
/// [`WINDOW_SAMPLES`] of audio has arrived (two seconds). That is what stops a
/// single utterance from firing on every overlapping window — no separate
/// cooldown timer is needed, and during those two seconds Handy is recording
/// anyway, so no audio reaches the spotter at all.
///
/// Deliberately free of threads, channels and clocks: `now` is passed in, so
/// the whole trigger policy is exercised synchronously by the tests below.
struct SpotterCore {
    detector: Box<dyn WakewordDetector>,
    window: Vec<f32>,
    since_last_score: usize,
    last_frame_at: Option<Instant>,
}

impl SpotterCore {
    fn new(detector: Box<dyn WakewordDetector>) -> Self {
        Self {
            detector,
            window: Vec::with_capacity(WINDOW_SAMPLES + HOP_SAMPLES),
            since_last_score: 0,
            last_frame_at: None,
        }
    }

    /// Feed one frame of 16 kHz audio. Returns `true` when it completes a window
    /// scoring at or above `threshold` — i.e. when the wake phrase fires.
    fn push(&mut self, frame: &[f32], threshold: f32, now: Instant) -> bool {
        // A gap means the buffered audio and this frame are not contiguous, so
        // scoring across the seam would be meaningless. Start the window over.
        if self
            .last_frame_at
            .is_some_and(|t| now.duration_since(t) > CONTINUITY_GAP)
        {
            self.reset_window();
        }
        self.last_frame_at = Some(now);

        self.since_last_score += frame.len();
        self.window.extend_from_slice(frame);
        if self.window.len() > WINDOW_SAMPLES {
            self.window.drain(..self.window.len() - WINDOW_SAMPLES);
        }

        if self.window.len() < WINDOW_SAMPLES || self.since_last_score < HOP_SAMPLES {
            return false;
        }
        self.since_last_score = 0;

        let pcm: Vec<i16> = self.window.iter().map(|s| to_i16(*s)).collect();
        let score = match self.detector.score(&pcm) {
            Ok(s) => s,
            Err(e) => {
                error!("Wake word scoring failed: {e}");
                return false;
            }
        };

        if score < threshold {
            debug!("Wake word window scored {score:.3}");
            return false;
        }

        info!("Wake word detected (score {score:.3} >= {threshold:.3})");
        self.reset_window();
        true
    }

    fn reset_window(&mut self) {
        self.window.clear();
        self.since_last_score = 0;
    }
}

/// Handle to the background spotter. Cloneable and cheap; dropping every clone
/// closes the channel and ends the worker thread.
#[derive(Clone)]
pub struct WakewordSpotter {
    tx: SyncSender<Vec<f32>>,
    enabled: Arc<AtomicBool>,
    /// `f32` threshold stored as raw bits so it can live in an atomic.
    threshold_bits: Arc<AtomicU32>,
}

impl WakewordSpotter {
    /// Start the worker thread. `on_detect` runs on that thread, so it should
    /// hand off rather than block.
    pub fn start(
        detector: Box<dyn WakewordDetector>,
        threshold: f32,
        enabled: bool,
        on_detect: Box<dyn Fn() + Send>,
    ) -> Self {
        let (tx, rx) = sync_channel::<Vec<f32>>(QUEUE_DEPTH);
        let enabled_flag = Arc::new(AtomicBool::new(enabled));
        let threshold_bits = Arc::new(AtomicU32::new(threshold.clamp(0.0, 1.0).to_bits()));

        let worker_threshold = Arc::clone(&threshold_bits);

        thread::Builder::new()
            .name("wakeword-spotter".into())
            .spawn(move || {
                let mut core = SpotterCore::new(detector);
                while let Ok(frame) = rx.recv() {
                    let threshold = f32::from_bits(worker_threshold.load(Ordering::Relaxed));
                    if core.push(&frame, threshold, Instant::now()) {
                        on_detect();
                    }
                }
                debug!("Wake word spotter thread exiting");
            })
            .expect("failed to spawn wake word spotter thread");

        Self {
            tx,
            enabled: enabled_flag,
            threshold_bits,
        }
    }

    /// Feed 16 kHz mono audio captured while Handy is idle. Called from the
    /// audio consumer thread — never blocks, and drops frames rather than
    /// stalling capture if the worker falls behind. A dropped frame shows up to
    /// the core as a continuity gap, which restarts the window.
    pub fn feed(&self, frame: &[f32]) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }
        match self.tx.try_send(frame.to_vec()) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                warn!("Wake word spotter queue full; dropping frame");
            }
            Err(TrySendError::Disconnected(_)) => {}
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn set_threshold(&self, threshold: f32) {
        self.threshold_bits
            .store(threshold.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }
}

/// Convert a normalised sample to signed 16-bit PCM, clamping rather than
/// wrapping so a hot microphone cannot alias loud speech into quiet noise.
fn to_i16(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
}

/// Classifier filenames looked for, in both the bundled resources and the app
/// data directory. Two renderings of the same phrase are shipped because the
/// English and Hebrew training pipelines produce noticeably different models;
/// whichever scores higher wins (see [`LiveKitDetector`]).
const CLASSIFIER_FILES: [&str; 2] = ["hey_claude_he.onnx", "hey_claude_en.onnx"];

/// Where a user drops their own classifier, overriding nothing — extra models
/// are simply additional voters.
pub fn user_model_dir(app: &tauri::AppHandle) -> anyhow::Result<std::path::PathBuf> {
    Ok(crate::portable::app_data_dir(app)
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .join("wakeword"))
}

/// Build the spotter and hand back a handle to keep in Tauri state.
///
/// Returns `None` when no classifier is installed — the feature is simply
/// absent then, rather than the app failing to start. The spotter is created
/// even when the setting is off so that toggling it takes effect without a
/// restart.
pub fn init(app: &tauri::AppHandle) -> Option<WakewordSpotter> {
    use tauri::Manager;

    let settings = crate::settings::get_settings(app);

    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    for name in CLASSIFIER_FILES {
        if let Ok(p) = app.path().resolve(
            format!("resources/wakeword/{name}"),
            tauri::path::BaseDirectory::Resource,
        ) {
            candidates.push(p);
        }
    }
    if let Ok(dir) = user_model_dir(app) {
        candidates.extend(CLASSIFIER_FILES.iter().map(|n| dir.join(n)));
    }

    let detector = match LiveKitDetector::new(&candidates) {
        Ok(d) => d,
        Err(e) => {
            info!("Wake word spotting unavailable: {e}");
            return None;
        }
    };

    // The spotter only ever sees audio while the microphone stream is open, so
    // an enabled wake word implies always-on capture. Persisted rather than
    // applied in memory so the settings UI shows the state the app is in.
    if settings.wakeword_enabled && !settings.always_on_microphone {
        info!("Wake word is enabled; turning on the always-on microphone it requires");
        let mut updated = settings.clone();
        updated.always_on_microphone = true;
        crate::settings::write_settings(app, updated);
    }

    let app_for_detect = app.clone();
    let spotter = WakewordSpotter::start(
        Box::new(detector),
        settings.wakeword_threshold,
        settings.wakeword_enabled,
        Box::new(move || {
            // Exactly what a shortcut press does, so the wake word inherits the
            // whole recording/transcription/paste pipeline unchanged.
            crate::signal_handle::send_transcription_input(
                &app_for_detect,
                "transcribe",
                "wakeword",
            );
        }),
    );

    info!(
        "Wake word spotting ready (enabled={}, threshold={:.2})",
        settings.wakeword_enabled, settings.wakeword_threshold
    );
    Some(spotter)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scores every window at a fixed value and counts the windows it saw.
    struct FixedDetector {
        score: f32,
        calls: Arc<AtomicU32>,
    }

    impl WakewordDetector for FixedDetector {
        fn score(&mut self, window: &[i16]) -> anyhow::Result<f32> {
            assert_eq!(window.len(), WINDOW_SAMPLES, "detector sees a full window");
            self.calls.fetch_add(1, Ordering::Relaxed);
            Ok(self.score)
        }
    }

    /// Fails every scoring attempt, standing in for a corrupt model file.
    struct FailingDetector;

    impl WakewordDetector for FailingDetector {
        fn score(&mut self, _window: &[i16]) -> anyhow::Result<f32> {
            Err(anyhow::anyhow!("model exploded"))
        }
    }

    /// The recorder's frame size: 30 ms at 16 kHz.
    const FRAME: usize = 480;

    struct Harness {
        core: SpotterCore,
        now: Instant,
        fires: usize,
    }

    impl Harness {
        fn new(detector: Box<dyn WakewordDetector>) -> Self {
            Self {
                core: SpotterCore::new(detector),
                now: Instant::now(),
                fires: 0,
            }
        }

        /// Feed `samples` worth of audio as contiguous real-time frames,
        /// advancing the clock exactly as the audio thread would.
        fn feed(&mut self, threshold: f32, samples: usize) {
            let frame = vec![0.1f32; FRAME];
            for _ in 0..samples.div_ceil(FRAME) {
                self.now += Duration::from_millis(30);
                if self.core.push(&frame, threshold, self.now) {
                    self.fires += 1;
                }
            }
        }

        /// Advance the clock without feeding audio, as happens while Handy is
        /// recording and the monitor callback is silent.
        fn idle(&mut self, gap: Duration) {
            self.now += gap;
        }
    }

    /// A detector plus the counter it increments, so a test can assert that
    /// windows were scored even when none of them fired.
    fn counted(score: f32) -> (Box<FixedDetector>, Arc<AtomicU32>) {
        let calls = Arc::new(AtomicU32::new(0));
        (
            Box::new(FixedDetector {
                score,
                calls: Arc::clone(&calls),
            }),
            calls,
        )
    }

    fn fixed(score: f32) -> Box<FixedDetector> {
        counted(score).0
    }

    #[test]
    fn partial_window_is_never_scored() {
        let mut h = Harness::new(fixed(0.99));
        h.feed(0.5, WINDOW_SAMPLES - FRAME);
        assert_eq!(h.fires, 0, "a partial window cannot fire");
    }

    #[test]
    fn full_window_above_threshold_fires_once() {
        let mut h = Harness::new(fixed(0.99));
        // Well past the first full window: without a cooldown every hop after it
        // would fire again.
        h.feed(0.5, WINDOW_SAMPLES + HOP_SAMPLES * 6);
        assert_eq!(h.fires, 1, "one phrase fires exactly once");
    }

    #[test]
    fn a_detection_costs_a_full_window_before_the_next_one() {
        let mut h = Harness::new(fixed(0.99));
        h.feed(0.5, WINDOW_SAMPLES);
        assert_eq!(h.fires, 1);

        // Anything short of a full fresh window cannot fire again, however
        // loudly it scores — the buffer was emptied by the detection.
        h.feed(0.5, WINDOW_SAMPLES - FRAME);
        assert_eq!(h.fires, 1, "a partial window after a detection is silent");

        // A second genuine utterance, once the window refills, does fire.
        h.feed(0.5, FRAME * 2);
        assert_eq!(h.fires, 2, "a later phrase fires again");
    }

    #[test]
    fn below_threshold_scores_but_never_fires() {
        let (detector, calls) = counted(0.49);
        let mut h = Harness::new(detector);
        h.feed(0.5, WINDOW_SAMPLES + HOP_SAMPLES * 3);

        assert_eq!(h.fires, 0, "0.49 does not clear a 0.5 threshold");
        assert!(
            calls.load(Ordering::Relaxed) >= 1,
            "windows were still scored"
        );
    }

    #[test]
    fn threshold_is_read_per_frame_not_captured_at_start() {
        let mut h = Harness::new(fixed(0.6));
        h.feed(0.9, WINDOW_SAMPLES + HOP_SAMPLES * 3);
        assert_eq!(h.fires, 0, "0.6 is below a 0.9 threshold");

        h.feed(0.5, HOP_SAMPLES * 2);
        assert_eq!(
            h.fires, 1,
            "lowering the threshold takes effect immediately"
        );
    }

    #[test]
    fn a_gap_restarts_the_window_instead_of_scoring_across_it() {
        let mut h = Harness::new(fixed(0.99));
        // Almost a full window, then the silence of a recording session.
        h.feed(0.5, WINDOW_SAMPLES - FRAME * 2);
        h.idle(CONTINUITY_GAP + Duration::from_millis(50));

        // One more frame would have completed the pre-gap window; it must not.
        h.feed(0.5, FRAME * 3);
        assert_eq!(h.fires, 0, "audio either side of a gap is not one window");

        // A fresh full window after the gap still works.
        h.feed(0.5, WINDOW_SAMPLES);
        assert_eq!(h.fires, 1, "spotting resumes after the gap");
    }

    #[test]
    fn detector_errors_are_swallowed_rather_than_firing() {
        let mut h = Harness::new(Box::new(FailingDetector));
        h.feed(0.5, WINDOW_SAMPLES + HOP_SAMPLES * 4);
        assert_eq!(h.fires, 0, "a broken model must never trigger recording");
    }

    #[test]
    fn clamps_rather_than_wrapping_hot_samples() {
        assert_eq!(to_i16(2.0), i16::MAX);
        assert_eq!(to_i16(-2.0), -i16::MAX);
        assert_eq!(to_i16(0.0), 0);
    }

    #[test]
    fn disabled_spotter_feeds_nothing_to_the_worker() {
        let spotter = WakewordSpotter::start(fixed(0.99), 0.5, false, Box::new(|| {}));
        assert!(!spotter.is_enabled());
        // Would panic in the worker's detector assert if a window ever formed.
        for _ in 0..200 {
            spotter.feed(&vec![0.1f32; FRAME]);
        }
        spotter.set_enabled(true);
        assert!(spotter.is_enabled());
    }
}
