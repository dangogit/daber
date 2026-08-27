pub mod audio;
pub mod history;
pub mod models;
pub mod transcription;

use crate::managers::{
    audio::{AudioRecordingManager, MicrophoneMode},
    history::HistoryManager,
    model::{ModelManager, IVRIT_MODEL_ID},
    transcription::TranscriptionManager,
};
use crate::settings::{get_settings, write_settings, AppSettings, LogLevel};
use crate::utils::cancel_current_operation;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
#[specta::specta]
pub fn cancel_operation(app: AppHandle) {
    cancel_current_operation(&app);
}

#[tauri::command]
#[specta::specta]
pub fn is_portable() -> bool {
    crate::portable::is_portable()
}

#[tauri::command]
#[specta::specta]
pub fn get_app_dir_path(app: AppHandle) -> Result<String, String> {
    let app_data_dir = crate::portable::app_data_dir(&app)
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    Ok(app_data_dir.to_string_lossy().to_string())
}

#[tauri::command]
#[specta::specta]
pub fn get_app_settings(app: AppHandle) -> Result<AppSettings, String> {
    Ok(get_settings(&app))
}

#[tauri::command]
#[specta::specta]
pub fn get_default_settings() -> Result<AppSettings, String> {
    Ok(crate::settings::get_default_settings())
}

fn validate_onboarding_completion(
    model_downloaded: bool,
    loaded_model: Option<&str>,
    has_successful_transcription: bool,
) -> Result<(), String> {
    if !model_downloaded {
        return Err("The Hebrew model has not finished downloading".to_string());
    }
    if loaded_model != Some(IVRIT_MODEL_ID) {
        return Err("The Hebrew model is not loaded".to_string());
    }
    if !has_successful_transcription {
        return Err("Complete one successful test dictation first".to_string());
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn complete_onboarding(
    app: AppHandle,
    model_manager: State<'_, Arc<ModelManager>>,
    transcription_manager: State<'_, Arc<TranscriptionManager>>,
    history_manager: State<'_, Arc<HistoryManager>>,
) -> Result<(), String> {
    let model_downloaded = model_manager
        .get_model_info(IVRIT_MODEL_ID)
        .is_some_and(|model| model.is_downloaded);
    let loaded_model = transcription_manager.get_current_model();
    let has_successful_transcription = history_manager
        .has_successful_transcription()
        .map_err(|error| format!("Failed to verify the test dictation: {error}"))?;
    validate_onboarding_completion(
        model_downloaded,
        loaded_model.as_deref(),
        has_successful_transcription,
    )?;

    let previous_settings = get_settings(&app);
    let mut settings = previous_settings.clone();
    settings.onboarding_completed = true;
    settings.always_on_microphone = true;
    write_settings(&app, settings);

    let recorder = app.state::<Arc<AudioRecordingManager>>().inner().clone();
    if let Err(error) =
        tokio::task::spawn_blocking(move || recorder.update_mode(MicrophoneMode::AlwaysOn))
            .await
            .map_err(|error| format!("Audio task failed: {error}"))?
    {
        write_settings(&app, previous_settings);
        return Err(format!("Failed to enable instant recording: {error}"));
    }

    Ok(())
}

/// Warm the microphone before the real onboarding dictation. The setting is
/// persisted only after a successful test, but the recorder can already keep a
/// short pre-roll so the first spoken word is present in that test too.
#[tauri::command]
#[specta::specta]
pub async fn prepare_onboarding_dictation(
    recorder: State<'_, Arc<AudioRecordingManager>>,
) -> Result<(), String> {
    let recorder = recorder.inner().clone();
    tokio::task::spawn_blocking(move || recorder.update_mode(MicrophoneMode::AlwaysOn))
        .await
        .map_err(|error| format!("Audio task failed: {error}"))?
        .map_err(|error| format!("Failed to prepare instant recording: {error}"))
}

#[cfg(test)]
mod onboarding_tests {
    use super::validate_onboarding_completion;

    #[test]
    fn onboarding_requires_model_load_and_real_dictation() {
        assert!(validate_onboarding_completion(false, None, false).is_err());
        assert!(validate_onboarding_completion(true, None, true).is_err());
        assert!(validate_onboarding_completion(true, Some("ivrit-turbo"), false).is_err());
        assert!(validate_onboarding_completion(true, Some("ivrit-turbo"), true).is_ok());
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_log_dir_path(app: AppHandle) -> Result<String, String> {
    let log_dir = crate::portable::app_log_dir(&app)
        .map_err(|e| format!("Failed to get log directory: {}", e))?;

    Ok(log_dir.to_string_lossy().to_string())
}

#[specta::specta]
#[tauri::command]
pub fn set_log_level(app: AppHandle, level: LogLevel) -> Result<(), String> {
    let tauri_log_level: tauri_plugin_log::LogLevel = level.into();
    let log_level: log::Level = tauri_log_level.into();
    // Update the file log level atomic so the filter picks up the new level
    crate::FILE_LOG_LEVEL.store(
        log_level.to_level_filter() as u8,
        std::sync::atomic::Ordering::Relaxed,
    );

    let mut settings = get_settings(&app);
    settings.log_level = level;
    write_settings(&app, settings);

    Ok(())
}

#[specta::specta]
#[tauri::command]
pub fn open_recordings_folder(app: AppHandle) -> Result<(), String> {
    let app_data_dir = crate::portable::app_data_dir(&app)
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let recordings_dir = app_data_dir.join("recordings");

    let path = recordings_dir.to_string_lossy().as_ref().to_string();
    app.opener()
        .open_path(path, None::<String>)
        .map_err(|e| format!("Failed to open recordings folder: {}", e))?;

    Ok(())
}

#[specta::specta]
#[tauri::command]
pub fn open_log_dir(app: AppHandle) -> Result<(), String> {
    let log_dir = crate::portable::app_log_dir(&app)
        .map_err(|e| format!("Failed to get log directory: {}", e))?;

    let path = log_dir.to_string_lossy().as_ref().to_string();
    app.opener()
        .open_path(path, None::<String>)
        .map_err(|e| format!("Failed to open log directory: {}", e))?;

    Ok(())
}

#[specta::specta]
#[tauri::command]
pub fn open_app_data_dir(app: AppHandle) -> Result<(), String> {
    let app_data_dir = crate::portable::app_data_dir(&app)
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let path = app_data_dir.to_string_lossy().as_ref().to_string();
    app.opener()
        .open_path(path, None::<String>)
        .map_err(|e| format!("Failed to open app data directory: {}", e))?;

    Ok(())
}

/// Check if Apple Intelligence is available on this device.
/// Called by the frontend when the user selects Apple Intelligence provider.
#[specta::specta]
#[tauri::command]
pub fn check_apple_intelligence_available() -> bool {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        crate::apple_intelligence::check_apple_intelligence_availability()
    }
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    {
        false
    }
}

/// Try to initialize Enigo (keyboard/mouse simulation).
/// On macOS, this will return an error if accessibility permissions are not granted.
#[specta::specta]
#[tauri::command]
pub fn initialize_enigo(app: AppHandle) -> Result<(), String> {
    use crate::input::EnigoState;

    // Check if already initialized
    if app.try_state::<EnigoState>().is_some() {
        log::debug!("Enigo already initialized");
        return Ok(());
    }

    // Try to initialize
    match EnigoState::new() {
        Ok(enigo_state) => {
            app.manage(enigo_state);
            log::info!("Enigo initialized successfully after permission grant");
            Ok(())
        }
        Err(e) => {
            if cfg!(target_os = "macos") {
                log::warn!(
                    "Failed to initialize Enigo: {} (accessibility permissions may not be granted)",
                    e
                );
            } else {
                log::warn!("Failed to initialize Enigo: {}", e);
            }
            Err(format!("Failed to initialize input system: {}", e))
        }
    }
}

/// Marker state to track if shortcuts have been initialized.
pub struct ShortcutsInitialized;

/// Initialize keyboard shortcuts.
/// On macOS, this should be called after accessibility permissions are granted.
/// This is idempotent - calling it multiple times is safe.
#[specta::specta]
#[tauri::command]
pub fn initialize_shortcuts(app: AppHandle) -> Result<(), String> {
    // Check if already initialized
    if app.try_state::<ShortcutsInitialized>().is_some() {
        log::debug!("Shortcuts already initialized");
        return Ok(());
    }

    // Initialize shortcuts
    crate::shortcut::init_shortcuts(&app);

    // Mark as initialized before reconciling the macOS Secure Input fallback.
    app.manage(ShortcutsInitialized);
    crate::secure_input::reconcile_fallback(&app);

    log::info!("Shortcuts initialized successfully");
    Ok(())
}

/// Match the native window appearance to the user's chosen theme.
///
/// On macOS the window is transparent with an `NSVisualEffectView` behind it,
/// and that view follows the *system* appearance rather than anything the CSS
/// says. Without this, forcing the light theme while the Mac is in dark mode
/// paints light surfaces over a dark vibrancy layer — washed out and low
/// contrast, which is not what "light" is supposed to look like.
///
/// `System` clears the override so the window follows the OS again.
#[specta::specta]
#[tauri::command]
pub fn set_window_theme(app: AppHandle, theme: crate::settings::Theme) -> Result<(), String> {
    let native = match theme {
        crate::settings::Theme::System => None,
        crate::settings::Theme::Light => Some(tauri::Theme::Light),
        crate::settings::Theme::Dark => Some(tauri::Theme::Dark),
    };

    for label in ["main", "overlay"] {
        if let Some(window) = app.get_webview_window(label) {
            if let Err(e) = window.set_theme(native) {
                log::warn!("Could not set {label} window theme: {e}");
            }
        }
    }

    Ok(())
}
