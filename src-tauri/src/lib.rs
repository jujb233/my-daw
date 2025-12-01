mod audio;
use audio::core::plugin::{NoteEvent, Plugin, PluginEvent};
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
    routing_track_index: usize,
}

#[derive(Clone, serde::Serialize)]
struct MixerTrackData {
    id: usize,
    label: String,
    volume: f32,
    meter_id: Option<Uuid>, // Added meter_id
}

struct AppState {
    audio_engine: Mutex<AudioEngine>,
    active_plugins: Mutex<Vec<PluginInstanceData>>,
    mixer_tracks: Mutex<Vec<MixerTrackData>>,
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

fn create_audio_graph(state: &State<'_, AppState>) -> Result<Box<dyn Plugin>, String> {
    let plugins = state
        .active_plugins
        .lock()
        .map_err(|_| "Failed to lock plugins list")?;

    let tracks = state
        .mixer_tracks
        .lock()
        .map_err(|_| "Failed to lock mixer tracks")?;

    let mut mixer = MixerPlugin::new(0);

    // Create configured tracks
    for track_data in tracks.iter() {
        mixer.add_track(track_data.meter_id);
    }

    // Let's refactor `create_audio_graph` to NOT take the lock, but take the data?
    // No, it needs to read plugins too.

    // Let's just ignore the meter ID persistence for a second.
    // The frontend needs the meter ID to look up the level.
    // If the ID changes every time, the frontend needs to know the new ID.
    // Maybe we can make the Meter ID deterministic? e.g. derived from Track ID?
    // `Uuid::new_v5`?

    // Let's try to make Meter ID deterministic based on Track ID.
    // We need to pass a seed or ID to `MixerTrack::new`.

    // In `src-tauri/src/audio/plugins/mixer/track.rs`:
    // Update `new` to take an ID?

    // Let's stick to the plan: Update `MixerTrackData` to have `meter_id`.
    // And update it when we rebuild.

    // Add Instruments to Mixer (Rack)
    for (i, p_data) in plugins.iter().enumerate() {
        if p_data.name == "SimpleSynth" {
            let synth = create_simple_synth();
            let inst_idx = mixer.add_instrument(synth);

            // Routing
            mixer.set_routing(inst_idx, p_data.routing_track_index);

            // Map params?
            // We need to map params on the MixerPlugin to the Instrument.
            // MixerPlugin needs `map_param`? It doesn't have it yet.
            // For now, let's skip param mapping or implement it in MixerPlugin.
        }
    }

    Ok(Box::new(mixer))
}

fn rebuild_engine(state: &State<'_, AppState>) -> Result<(), String> {
    let mut engine = state
        .audio_engine
        .lock()
        .map_err(|_| "Failed to lock audio engine")?;

    if engine.is_running() {
        engine.stop();
        let root = create_audio_graph(state)?;
        engine.start(root).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn add_mixer_track(state: State<'_, AppState>) -> Result<(), String> {
    {
        let mut tracks = state
            .mixer_tracks
            .lock()
            .map_err(|_| "Failed to lock tracks")?;
        let id = tracks.len();
        tracks.push(MixerTrackData {
            id,
            label: format!("Track {}", id + 1),
            volume: 1.0,
            meter_id: Some(Uuid::new_v4()), // Generate ID here
        });
    }
    rebuild_engine(&state)?;
    Ok(())
}

#[tauri::command]
fn remove_mixer_track(state: State<'_, AppState>, index: usize) -> Result<(), String> {
    {
        let mut tracks = state
            .mixer_tracks
            .lock()
            .map_err(|_| "Failed to lock tracks")?;
        if index < tracks.len() {
            tracks.remove(index);
            // Re-index
            for (i, track) in tracks.iter_mut().enumerate() {
                track.id = i;
            }
        }
    }
    rebuild_engine(&state)?;
    Ok(())
}

#[tauri::command]
fn get_mixer_tracks(state: State<'_, AppState>) -> Result<Vec<MixerTrackData>, String> {
    let tracks = state
        .mixer_tracks
        .lock()
        .map_err(|_| "Failed to lock tracks")?;
    Ok(tracks.clone())
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
            routing_track_index: 0,
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
        let root = create_audio_graph(&state)?;

        engine.start(root).map_err(|e| e.to_string())?;

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

#[tauri::command]
fn set_instrument_routing(
    state: State<'_, AppState>,
    inst_index: usize,
    track_index: usize,
) -> Result<(), String> {
    {
        let mut plugins = state
            .active_plugins
            .lock()
            .map_err(|_| "Failed to lock plugins list")?;

        if let Some(inst) = plugins.get_mut(inst_index) {
            inst.routing_track_index = track_index;
        }
    }
    rebuild_engine(&state)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize with 5 default tracks
    let mut tracks = Vec::new();
    for i in 0..5 {
        tracks.push(MixerTrackData {
            id: i,
            label: format!("Track {}", i + 1),
            volume: 1.0,
            meter_id: Some(Uuid::new_v4()),
        });
    }

    tauri::Builder::default()
        .manage(AppState {
            audio_engine: Mutex::new(AudioEngine::new()),
            active_plugins: Mutex::new(Vec::new()),
            mixer_tracks: Mutex::new(tracks),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            toggle_audio,
            update_parameter,
            add_plugin_instance,
            remove_plugin_instance,
            update_plugin_label,
            get_meter_levels_cmd,
            add_mixer_track,
            remove_mixer_track,
            get_mixer_tracks,
            set_instrument_routing
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
