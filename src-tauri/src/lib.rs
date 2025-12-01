mod audio;
use audio::engine::AudioEngine;
use audio::nodes::sine::SineWave;
use std::sync::Mutex;
use tauri::State;

struct AppState {
    audio_engine: Mutex<AudioEngine>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn toggle_audio(state: State<'_, AppState>) -> Result<bool, String> {
    let mut engine = state.audio_engine.lock().map_err(|_| "Failed to lock audio engine")?;
    
    if engine.is_running() {
        engine.stop();
        Ok(false)
    } else {
        let sine = SineWave::new(440.0); // A4
        engine.start(Box::new(sine)).map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            audio_engine: Mutex::new(AudioEngine::new()),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, toggle_audio])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
