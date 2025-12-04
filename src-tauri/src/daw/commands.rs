use super::core::{create_audio_graph, rebuild_engine};
use super::state::{AppState, MixerTrackData, PluginInstanceData};
use crate::audio::core::plugin::{NoteEvent, PluginEvent};
use crate::audio::plugins::mixer::level_meter::get_meter_levels;
use crate::daw::sequencer::{get_is_playing, get_playback_position};
use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn get_playback_state() -> (bool, f64) {
    (get_is_playing(), get_playback_position())
}

#[tauri::command]
pub fn get_meter_levels_cmd() -> HashMap<Uuid, f32> {
    get_meter_levels()
}

#[tauri::command]
pub fn add_mixer_track(state: State<'_, AppState>) -> Result<(), String> {
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
            pan: 0.0,
            mute: false,
            solo: false,
            meter_id: Some(Uuid::new_v4()), // 在这里生成 ID
        });
    }
    rebuild_engine(&state)?;
    Ok(())
}

#[tauri::command]
pub fn remove_mixer_track(state: State<'_, AppState>, index: usize) -> Result<(), String> {
    // 0 号轨道是总轨，不可删除
    if index == 0 {
        return Err("Cannot remove Master track".to_string());
    }

    {
        let mut tracks = state
            .mixer_tracks
            .lock()
            .map_err(|_| "Failed to lock tracks")?;
        if index < tracks.len() {
            tracks.remove(index);
            // 重新索引
            for (i, track) in tracks.iter_mut().enumerate() {
                track.id = i;
            }
        }
    }
    rebuild_engine(&state)?;
    Ok(())
}

#[tauri::command]
pub fn get_mixer_tracks(state: State<'_, AppState>) -> Result<Vec<MixerTrackData>, String> {
    let tracks = state
        .mixer_tracks
        .lock()
        .map_err(|_| "Failed to lock tracks")?;
    Ok(tracks.clone())
}

#[tauri::command]
pub fn get_active_plugins(state: State<'_, AppState>) -> Result<Vec<PluginInstanceData>, String> {
    let plugins = state
        .active_plugins
        .lock()
        .map_err(|_| "Failed to lock plugins")?;
    Ok(plugins.clone())
}

#[tauri::command]
pub fn add_plugin_instance(state: State<'_, AppState>, name: String) -> Result<(), String> {
    {
        let mut plugins = state
            .active_plugins
            .lock()
            .map_err(|_| "Failed to lock plugins list")?;

        plugins.push(PluginInstanceData {
            id: Uuid::new_v4().to_string(),
            name,
            label: "New Instrument".to_string(),
            routing_track_index: 0,
        });
    } // 解锁插件

    rebuild_engine(&state)?;

    Ok(())
}

#[tauri::command]
pub fn remove_plugin_instance(state: State<'_, AppState>, index: usize) -> Result<(), String> {
    {
        let mut plugins = state
            .active_plugins
            .lock()
            .map_err(|_| "Failed to lock plugins list")?;

        if index < plugins.len() {
            plugins.remove(index);
        }
    } // 解锁插件

    rebuild_engine(&state)?;

    Ok(())
}

#[tauri::command]
pub fn update_plugin_label(
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
pub fn toggle_audio(state: State<'_, AppState>) -> Result<bool, String> {
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

        // 立即向所有合成器发送测试音符？
        // 目前我们使用广播 MIDI 的方式。
        engine.send_event(PluginEvent::Midi(NoteEvent::NoteOn {
            note: 69,
            velocity: 1.0,
        })); // A4

        Ok(true)
    }
}

#[tauri::command]
pub fn update_parameter(
    state: State<'_, AppState>,
    param_id: u32,
    value: f32,
) -> Result<(), String> {
    // println!("Command: update_parameter {} {}", param_id, value); // 太嘈杂？
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
pub fn set_instrument_routing(
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

#[tauri::command]
pub fn play(state: State<'_, AppState>) -> Result<(), String> {
    let mut engine = state
        .audio_engine
        .lock()
        .map_err(|_| "Failed to lock audio engine")?;

    if !engine.is_running() {
        // 启动引擎
        let root = create_audio_graph(&state)?;

        engine.start(root).map_err(|e| e.to_string())?;
    }

    engine.send_event(PluginEvent::Transport {
        playing: true,
        position: None,
        tempo: None,
    });
    Ok(())
}
#[tauri::command]
pub fn pause(state: State<'_, AppState>) -> Result<(), String> {
    let engine = state
        .audio_engine
        .lock()
        .map_err(|_| "Failed to lock audio engine")?;

    if engine.is_running() {
        engine.send_event(PluginEvent::Transport {
            playing: false,
            position: None,
            tempo: None,
        });
    }
    Ok(())
}

#[tauri::command]
pub fn stop(state: State<'_, AppState>) -> Result<(), String> {
    let engine = state
        .audio_engine
        .lock()
        .map_err(|_| "Failed to lock audio engine")?;

    if engine.is_running() {
        engine.send_event(PluginEvent::Transport {
            playing: false,
            position: Some(0.0),
            tempo: None,
        });
    }
    Ok(())
}

#[tauri::command]
pub fn seek(state: State<'_, AppState>, position: f64) -> Result<(), String> {
    let engine = state
        .audio_engine
        .lock()
        .map_err(|_| "Failed to lock audio engine")?;

    if engine.is_running() {
        // 我们保留播放状态
        let playing = get_is_playing();
        engine.send_event(PluginEvent::Transport {
            playing,
            position: Some(position),
            tempo: None,
        });
    }
    Ok(())
}
