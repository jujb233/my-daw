mod audio;
use audio::engine::AudioEngine;
use audio::nodes::container::PluginContainer;
use audio::nodes::gain_fader::GainFader;
use audio::nodes::wave_generator::WaveGenerator;
use audio::plugin::{NoteEvent, PluginEvent};
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
fn toggle_audio(state: State<'_, AppState>, initial_gain: f32) -> Result<bool, String> {
    let mut engine = state
        .audio_engine
        .lock()
        .map_err(|_| "Failed to lock audio engine")?;

    if engine.is_running() {
        engine.stop();
        Ok(false)
    } else {
        // Build the plugin chain: WaveGenerator -> GainFader
        let mut container = PluginContainer::new();

        let mut wave = WaveGenerator::new();
        // wave.active = true; // We need to trigger a note on event now

        let gain = GainFader::new(); // Default gain 0.5

        container.add_plugin(Box::new(wave));
        container.add_plugin(Box::new(gain));

        engine
            .start(Box::new(container))
            .map_err(|e| e.to_string())?;

        // Send initial gain
        engine.send_event(PluginEvent::Parameter {
            id: 0,
            value: initial_gain,
        });

        // Send a test note immediately
        engine.send_event(PluginEvent::Midi(NoteEvent::NoteOn {
            note: 69,
            velocity: 1.0,
        })); // A4

        Ok(true)
    }
}

#[tauri::command]
fn update_parameter(state: State<'_, AppState>, param_id: u32, value: f32) -> Result<(), String> {
    let engine = state
        .audio_engine
        .lock()
        .map_err(|_| "Failed to lock audio engine")?;

    if engine.is_running() {
        engine.send_event(PluginEvent::Parameter {
            id: param_id,
            value,
        });
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            audio_engine: Mutex::new(AudioEngine::new()),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            toggle_audio,
            update_parameter
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
