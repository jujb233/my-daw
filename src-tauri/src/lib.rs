mod audio;
mod daw;

use crate::audio::engine::AudioEngine;
use crate::audio::plugins::manager::PluginManager;
use daw::clip_commands::*;
use daw::commands::*;
use daw::model::ArrangementTrack;
use daw::state::{AppState, MixerTrackData, PluginInstanceData};
use daw::track_commands::*;
use std::sync::Mutex;
use uuid::Uuid;

#[tauri::command]
fn get_available_plugins(
    state: tauri::State<AppState>,
) -> Vec<crate::audio::core::plugin::PluginInfo> {
    let manager = state.plugin_manager.lock().unwrap();
    manager.get_available_plugins()
}

#[tauri::command]
fn get_plugin_parameters(
    state: tauri::State<AppState>,
    unique_id: String,
) -> Option<Vec<crate::audio::core::plugin::PluginParameter>> {
    let manager = state.plugin_manager.lock().unwrap();
    manager.get_plugin_parameters(&unique_id)
}

#[tauri::command]
fn import_plugin(
    state: tauri::State<AppState>,
    path: String,
) -> Result<crate::audio::core::plugin::PluginInfo, String> {
    let mut manager = state.plugin_manager.lock().unwrap();
    manager.scan_clap_plugin(&path)
}

#[tauri::command]
fn log_msg(msg: String) {
    println!("Frontend Log: {}", msg);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 使用 5 个默认轨道初始化
    let mut tracks = Vec::new();
    // Track 0 is Master
    tracks.push(MixerTrackData {
        id: 0,
        label: "Master".to_string(),
        volume: 1.0,
        pan: 0.0,
        mute: false,
        solo: false,
        meter_id: Some(Uuid::new_v4()),
    });

    for i in 1..5 {
        tracks.push(MixerTrackData {
            id: i,
            label: format!("Track {}", i),
            volume: 1.0,
            pan: 0.0,
            mute: false,
            solo: false,
            meter_id: Some(Uuid::new_v4()),
        });
    }

    // 使用默认的 SimpleSynth 初始化
    let mut plugins = Vec::new();
    plugins.push(PluginInstanceData {
        id: Uuid::new_v4().to_string(),
        name: "com.mydaw.simplesynth".to_string(),
        label: "Grand Piano".to_string(),
        routing_track_index: 0,
    });

    // 初始化编排轨道 (Arrangement Tracks)
    // 默认创建 4 个编排轨道，分别路由到 Mixer Track 1, 2, 3, 4
    // 注意：Mixer Track 0 是 Master
    let mut arrangement_tracks = Vec::new();
    for i in 0..4 {
        arrangement_tracks.push(ArrangementTrack {
            id: i,
            name: format!("Track {}", i + 1),
            color: "#aec6ff".to_string(),
            muted: false,
            soloed: false,
            target_mixer_track_id: i + 1, // 默认路由到对应的 Mixer Track
        });
    }

    tauri::Builder::default()
        .manage(AppState {
            audio_engine: Mutex::new(AudioEngine::new()),
            plugin_manager: Mutex::new(PluginManager::new()),
            active_plugins: Mutex::new(plugins),
            mixer_tracks: Mutex::new(tracks),
            arrangement_tracks: Mutex::new(arrangement_tracks),
            clips: Mutex::new(Vec::new()),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
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
            set_instrument_routing,
            get_active_plugins,
            add_clip,
            update_clip,
            copy_clip,
            get_clip,
            remove_clip,
            play,
            get_playback_state,
            pause,
            stop,
            seek,
            get_available_plugins,
            get_plugin_parameters,
            import_plugin,
            get_arrangement_tracks,
            add_arrangement_track,
            remove_arrangement_track,
            log_msg
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
