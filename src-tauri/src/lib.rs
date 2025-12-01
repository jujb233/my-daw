mod audio;
use audio::core::plugin::{NoteEvent, PluginEvent};
use audio::engine::AudioEngine;
use audio::plugins::container::PluginContainer;
use audio::plugins::simple_synth::create_simple_synth;
use std::sync::Mutex;
use tauri::State;

struct AppState {
    audio_engine: Mutex<AudioEngine>,
    active_plugins: Mutex<Vec<String>>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn add_plugin_instance(state: State<'_, AppState>, name: String) -> Result<(), String> {
    let mut plugins = state
        .active_plugins
        .lock()
        .map_err(|_| "Failed to lock plugins list")?;

    plugins.push(name);

    // If engine is running, we should ideally restart it or update it.
    // For this prototype, we'll require a toggle off/on to take effect,
    // OR we can force a restart here.
    // Let's force a restart if running.
    let mut engine = state
        .audio_engine
        .lock()
        .map_err(|_| "Failed to lock audio engine")?;

    if engine.is_running() {
        engine.stop();

        // Rebuild
        let mut container = PluginContainer::new();
        for (i, p_name) in plugins.iter().enumerate() {
            if p_name == "SimpleSynth" {
                let synth = create_simple_synth();
                let idx = container.add_plugin(synth);
                // Map params: 2 params per synth
                // Global ID = i * 2 + local_id
                container.map_param((i * 2) as u32, idx, 0); // Gain
                container.map_param((i * 2 + 1) as u32, idx, 1); // Waveform
            }
        }

        engine
            .start(Box::new(container))
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn toggle_audio(state: State<'_, AppState>) -> Result<bool, String> {
    let mut engine = state
        .audio_engine
        .lock()
        .map_err(|_| "Failed to lock audio engine")?;

    if engine.is_running() {
        engine.stop();
        Ok(false)
    } else {
        let plugins = state
            .active_plugins
            .lock()
            .map_err(|_| "Failed to lock plugins list")?;

        // Build the plugin chain
        let mut container = PluginContainer::new();

        if plugins.is_empty() {
            // Default fallback if empty? Or just silence.
        }

        for (i, p_name) in plugins.iter().enumerate() {
            if p_name == "SimpleSynth" {
                let synth = create_simple_synth();
                let idx = container.add_plugin(synth);
                // Map params: 2 params per synth
                // Global ID = i * 2 + local_id
                container.map_param((i * 2) as u32, idx, 0); // Gain
                container.map_param((i * 2 + 1) as u32, idx, 1); // Waveform
            }
        }

        engine
            .start(Box::new(container))
            .map_err(|e| e.to_string())?;

        // Send a test note immediately to ALL synths?
        // Currently we broadcast MIDI.
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
            active_plugins: Mutex::new(Vec::new()),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            toggle_audio,
            update_parameter,
            add_plugin_instance
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
