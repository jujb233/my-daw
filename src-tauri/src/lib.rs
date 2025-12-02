mod audio;
mod daw;

use crate::audio::engine::AudioEngine;
use daw::clip_commands::*;
use daw::commands::*;
use daw::state::{AppState, MixerTrackData};
use std::sync::Mutex;
use uuid::Uuid;

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
            clips: Mutex::new(Vec::new()),
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
            set_instrument_routing,
            add_clip,
            update_clip,
            get_clip,
            play,
            pause,
            stop,
            seek,
            get_playback_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
