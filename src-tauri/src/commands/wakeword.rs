use crate::managers::audio::{AudioRecordingManager, MicrophoneMode};
use crate::settings::{get_settings, write_settings};
use crate::wakeword::WakewordSpotter;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

/// Whether a wake word classifier is installed. When false the feature cannot
/// be turned on, and the settings UI says so rather than offering a dead toggle.
#[tauri::command]
#[specta::specta]
pub fn is_wakeword_available(app: AppHandle) -> bool {
    app.try_state::<WakewordSpotter>().is_some()
}

/// Turn wake word spotting on or off at runtime.
///
/// Enabling also switches the microphone to always-on: the spotter only ever
/// sees audio while the stream is open. Disabling deliberately leaves the
/// microphone mode alone — the user may have chosen always-on for its own sake,
/// and silently reverting it would be surprising.
#[tauri::command]
#[specta::specta]
pub async fn update_wakeword_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    let spotter = app
        .try_state::<WakewordSpotter>()
        .ok_or_else(|| "No wake word classifier is installed".to_string())?
        .inner()
        .clone();

    let mut settings = get_settings(&app);
    settings.wakeword_enabled = enabled;
    let needs_always_on = enabled && !settings.always_on_microphone;
    if needs_always_on {
        settings.always_on_microphone = true;
    }
    write_settings(&app, settings);

    spotter.set_enabled(enabled);

    if needs_always_on {
        // Opening the cpal stream blocks on CoreAudio and takes the manager's
        // mutexes; keep it off the webview/main run loop (see
        // `update_microphone_mode`).
        let rm = app.state::<Arc<AudioRecordingManager>>().inner().clone();
        tokio::task::spawn_blocking(move || rm.update_mode(MicrophoneMode::AlwaysOn))
            .await
            .map_err(|e| format!("audio task join failed: {e}"))?
            .map_err(|e| format!("Failed to enable the always-on microphone: {e}"))?;
    }

    Ok(())
}

/// Set the confidence a window must reach to fire, in `0.0..=1.0`. Applies to
/// the running spotter immediately.
#[tauri::command]
#[specta::specta]
pub fn update_wakeword_threshold(app: AppHandle, threshold: f32) -> Result<(), String> {
    if !(0.0..=1.0).contains(&threshold) {
        return Err(format!(
            "Threshold must be between 0 and 1, got {threshold}"
        ));
    }

    let mut settings = get_settings(&app);
    settings.wakeword_threshold = threshold;
    write_settings(&app, settings);

    if let Some(spotter) = app.try_state::<WakewordSpotter>() {
        spotter.set_threshold(threshold);
    }

    Ok(())
}
