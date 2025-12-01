mod audio;
use audio::core::plugin::{NoteEvent, PluginEvent};
use audio::engine::AudioEngine;
use audio::plugins::container::PluginContainer;
use audio::plugins::mixer::level_meter::get_meter_levels;
use audio::plugins::mixer::mixer_plugin::MixerPlugin;
use audio::plugins::simple_synth::create_simple_synth;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

struct PluginInstanceData {
    name: String,
    label: String,
}

struct AppState {
    audio_engine: Mutex<AudioEngine>,
    active_plugins: Mutex<Vec<PluginInstanceData>>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_meter_levels_cmd() -> HashMap<Uuid, f32> {
    get_meter_levels()
}

fn rebuild_engine(state: &State<'_, AppState>) -> Result<(), String> {
    let mut engine = state
        .audio_engine
        .lock()
        .map_err(|_| "Failed to lock audio engine")?;

    if engine.is_running() {
        engine.stop();

        let plugins = state
            .active_plugins
            .lock()
            .map_err(|_| "Failed to lock plugins list")?;

        // Rebuild with Mixer
        let mut mixer = MixerPlugin::new(0);

        for (i, p_data) in plugins.iter().enumerate() {
            mixer.add_track();
            if let Some(track) = mixer.get_track_mut(i) {
                if p_data.name == "SimpleSynth" {
                    let synth = create_simple_synth();
                    track.container.insert_plugin(0, synth);

                    // Map params
                    // Track Fader is at Param 0 (mapped in MixerTrack::new)
                    // Synth Gain -> Param 10
                    // Synth Wave -> Param 11
                    track.container.map_param(10, 0, 0);
                    track.container.map_param(11, 0, 1);
                }
            }
        }

        engine.start(Box::new(mixer)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn add_plugin_instance(state: State<'_, AppState>, name: String) -> Result<(), String> {
    {
        let mut plugins = state
            .active_plugins
            .lock()
            .map_err(|_| "Failed to lock plugins list")?;

        plugins.push(PluginInstanceData {
            name,
            label: "New Instrument".to_string(),
        });
    } // Unlock plugins

    rebuild_engine(&state)?;

    Ok(())
}

#[tauri::command]
fn remove_plugin_instance(state: State<'_, AppState>, index: usize) -> Result<(), String> {
    {
        let mut plugins = state
            .active_plugins
            .lock()
            .map_err(|_| "Failed to lock plugins list")?;

        if index < plugins.len() {
            plugins.remove(index);
        }
    } // Unlock plugins

    rebuild_engine(&state)?;

    Ok(())
}

#[tauri::command]
fn update_plugin_label(
    state: State<'_, AppState>,
    index: usize,
    label: String,
) -> Result<(), String> {
    let mut plugins = state
        .active_plugins
        .lock()
        .map_err(|_| "Failed to lock plugins list")?;

    if let Some(plugin) = plugins.get_mut(index) {
        plugin.label = label;
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

        for (i, p_data) in plugins.iter().enumerate() {
            if p_data.name == "SimpleSynth" {
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
            add_plugin_instance,
            remove_plugin_instance,
            update_plugin_label,
            get_meter_levels_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
