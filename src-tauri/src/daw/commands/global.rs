use crate::audio::core::plugin::{NoteEvent, ParameterType, PluginEvent, PluginParameter};
use crate::audio::plugins::mixer::level_meter::get_meter_levels;
/// 全局 Tauri 命令：播放控制、轨道/插件管理与项目保存/加载（通过 AppState/Engine 操作）
use crate::daw::core::{create_audio_graph, rebuild_engine};
use crate::daw::sequencer::{get_is_playing, get_playback_position};
use crate::daw::serialization::project::ProjectManager;
use crate::daw::state::{AppState, MixerTrackData, PluginInstanceData};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn greet(name: &str) -> String {
        // 简单示例命令，用于测试 IPC 通道
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
                let mut tracks = state.mixer_tracks.lock().map_err(|_| "Failed to lock tracks")?;
                let id = tracks.len();
                tracks.push(MixerTrackData {
                        id,
                        label: format!("Track {}", id + 1),
                        volume: 1.0,
                        pan: 0.0,
                        mute: false,
                        solo: false,
                        meter_id: Some(Uuid::new_v4()), // 生成电平表 ID
                });
        }
        rebuild_engine(&state)?;
        Ok(())
}

#[tauri::command]
pub fn remove_mixer_track(state: State<'_, AppState>, index: usize) -> Result<(), String> {
        // Master 轨（0）不可删除
        if index == 0 {
                return Err("Cannot remove Master track".to_string());
        }

        {
                let mut tracks = state.mixer_tracks.lock().map_err(|_| "Failed to lock tracks")?;
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
        let tracks = state.mixer_tracks.lock().map_err(|_| "Failed to lock tracks")?;
        Ok(tracks.clone())
}

#[tauri::command]
pub fn get_active_plugins(state: State<'_, AppState>) -> Result<Vec<PluginInstanceData>, String> {
        let plugins = state.active_plugins.lock().map_err(|_| "Failed to lock plugins")?;
        Ok(plugins.clone())
}

#[tauri::command]
pub fn add_plugin_instance(state: State<'_, AppState>, name: String) -> Result<(), String> {
        {
                let mut plugins = state.active_plugins.lock().map_err(|_| "Failed to lock plugins list")?;

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
                let mut plugins = state.active_plugins.lock().map_err(|_| "Failed to lock plugins list")?;

                if index < plugins.len() {
                        plugins.remove(index);
                }
        } // 解锁插件

        rebuild_engine(&state)?;

        Ok(())
}

#[tauri::command]
pub fn update_plugin_label(state: State<'_, AppState>, index: usize, label: String) -> Result<(), String> {
        let mut plugins = state.active_plugins.lock().map_err(|_| "Failed to lock plugins list")?;

        if let Some(plugin) = plugins.get_mut(index) {
                plugin.label = label;
        }
        Ok(())
}

#[tauri::command]
pub fn toggle_audio(state: State<'_, AppState>) -> Result<bool, String> {
        let mut engine = state.audio_engine.lock().map_err(|_| "Failed to lock audio engine")?;

        if engine.is_running() {
                engine.stop();
                Ok(false)
        } else {
                let (root, _instances) = create_audio_graph(&state)?;

                engine.start(root).map_err(|e| e.to_string())?;

                // 启动后发送测试音符以验证音频路径
                engine.send_event(PluginEvent::Midi(NoteEvent::NoteOn {
                        note: 69,
                        velocity: 1.0,
                })); // A4

                Ok(true)
        }
}

#[tauri::command]
pub fn update_parameter(state: State<'_, AppState>, param_id: u32, value: f32) -> Result<(), String> {
        // 发送参数更新事件（若音频引擎运行）
        let engine = state.audio_engine.lock().map_err(|_| "Failed to lock audio engine")?;

        if engine.is_running() {
                engine.send_event(PluginEvent::Parameter { id: param_id, value });
        }
        Ok(())
}

#[derive(Serialize)]
pub struct ParamsWithValues {
        pub params: Vec<crate::audio::core::plugin::PluginParameter>,
        pub values: Vec<f32>,
}

#[tauri::command]
pub fn get_instance_parameters(
        state: State<'_, AppState>,
        instance_id: String,
) -> Result<Option<ParamsWithValues>, String> {
        let instances = state
                .plugin_instances
                .lock()
                .map_err(|_| "Failed to lock plugin instances")?;

        if let Some(inst_arc) = instances.get(&instance_id) {
                if let Ok(inst) = inst_arc.lock() {
                        let mut params = inst.get_parameters();
                        if params.is_empty() {
                                if let Some(fallback) = default_params_for_id(&inst.info().unique_id) {
                                        params = fallback;
                                }
                        }
                        let mut values = Vec::new();
                        for p in &params {
                                values.push(inst.get_param(p.id));
                        }
                        return Ok(Some(ParamsWithValues { params, values }));
                }
        }
        Ok(None)
}

#[tauri::command]
pub fn set_instance_parameter(
        state: State<'_, AppState>,
        instance_id: String,
        param_id: u32,
        value: f32,
) -> Result<(), String> {
        let instances = state
                .plugin_instances
                .lock()
                .map_err(|_| "Failed to lock plugin instances")?;

        if let Some(inst_arc) = instances.get(&instance_id) {
                if let Ok(mut inst) = inst_arc.lock() {
                        inst.set_param(param_id, value);
                        return Ok(());
                }
        }
        Err("Instance not found".to_string())
}

fn default_params_for_id(id: &str) -> Option<Vec<PluginParameter>> {
        match id {
                "com.mydaw.gainfader" => Some(vec![PluginParameter {
                        id: 0,
                        name: "Gain (dB)".to_string(),
                        min_value: -60.0,
                        max_value: 12.0,
                        default_value: 0.0,
                        value_type: ParameterType::Float,
                }]),
                "com.mydaw.simplesynth" => Some(vec![PluginParameter {
                        id: 0,
                        name: "Frequency".to_string(),
                        min_value: 20.0,
                        max_value: 2000.0,
                        default_value: 440.0,
                        value_type: ParameterType::Float,
                }]),
                // Wave generator currently exposes no parameters
                _ => None,
        }
}

#[tauri::command]
pub fn set_instrument_routing(state: State<'_, AppState>, inst_index: usize, track_index: usize) -> Result<(), String> {
        {
                let mut plugins = state.active_plugins.lock().map_err(|_| "Failed to lock plugins list")?;

                if let Some(inst) = plugins.get_mut(inst_index) {
                        inst.routing_track_index = track_index;
                }
        }
        rebuild_engine(&state)?;
        Ok(())
}

#[tauri::command]
pub fn play(state: State<'_, AppState>) -> Result<(), String> {
        let mut engine = state.audio_engine.lock().map_err(|_| "Failed to lock audio engine")?;

        if !engine.is_running() {
                // 启动引擎
                let (root, _instances) = create_audio_graph(&state)?;

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
        let engine = state.audio_engine.lock().map_err(|_| "Failed to lock audio engine")?;

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
        let engine = state.audio_engine.lock().map_err(|_| "Failed to lock audio engine")?;

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
        let engine = state.audio_engine.lock().map_err(|_| "Failed to lock audio engine")?;

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

#[tauri::command]
pub fn save_project_cmd(state: State<'_, AppState>, path: String) -> Result<(), String> {
        // 封装 ProjectManager 保存入口
        let path_buf = PathBuf::from(path);
        ProjectManager::save_project(&state, &path_buf).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_project_cmd(state: State<'_, AppState>, path: String) -> Result<(), String> {
        let path_buf = PathBuf::from(path);
        let schema = ProjectManager::load_project(&path_buf).map_err(|e| e.to_string())?;
        let plugin_states = ProjectManager::load_plugin_states(&path_buf).map_err(|e| e.to_string())?;

        // Ensure plugin manager registers plugins found inside the opened project so
        // they show up in the available plugin list and can be instantiated. Project
        // copies in `project/plugins` should take precedence when present.
        {
                let mut mgr = state.plugin_manager.lock().map_err(|_| "Lock error")?;
                let plugins_dir = path_buf.join("plugins");
                if plugins_dir.exists() {
                        mgr.scan_plugins_dir(&plugins_dir);
                }
        }

        // Apply schema to state
        {
                let mut tracks = state.arrangement_tracks.lock().map_err(|_| "Lock error")?;
                tracks.clear();
                for t in schema.tracks.iter() {
                        tracks.push(crate::daw::model::ArrangementTrack {
                                id: t.id,
                                name: t.name.clone(),
                                color: t.color.clone(),
                                muted: false,
                                soloed: false,
                                target_mixer_track_id: t.target_mixer_track_id,
                        });
                }
        }

        {
                let mut clips = state.clips.lock().map_err(|_| "Lock error")?;
                clips.clear();
                for t in schema.tracks.iter() {
                        for c in &t.clips {
                                let content = match &c.content_type {
                                        crate::daw::serialization::schema::ClipContentType::Midi => {
                                                crate::daw::model::ClipContent::Midi
                                        }
                                        crate::daw::serialization::schema::ClipContentType::Audio { file_path } => {
                                                crate::daw::model::ClipContent::Audio {
                                                        path: file_path.clone(),
                                                }
                                        }
                                };

                                clips.push(crate::daw::model::Clip {
                                        id: c.id.clone(),
                                        track_id: t.id,
                                        name: c.name.clone(),
                                        color: c.color.clone(),
                                        start: crate::daw::model::Position {
                                                bar: c.start_bar,
                                                beat: c.start_beat,
                                                sixteenth: c.start_sixteenth,
                                                tick: c.start_tick,
                                                time: c.start_position,
                                        },
                                        length: crate::daw::model::MusicalLength {
                                                bars: c.duration_bars,
                                                beats: c.duration_beats,
                                                sixteenths: c.duration_sixteenths,
                                                ticks: c.duration_ticks,
                                                total_ticks: c.duration_total_ticks,
                                                seconds: c.duration,
                                        },
                                        notes: c.notes
                                                .iter()
                                                .map(|n| {
                                                        let bpm = schema.settings.bpm;
                                                        let (num, _den) = schema.settings.time_signature;
                                                        let ppq = 960.0;

                                                        let seconds_per_beat = 60.0 / bpm;
                                                        let total_ticks =
                                                                (n.start / seconds_per_beat * ppq).round() as u64;

                                                        let ticks_per_beat = ppq as u64;
                                                        let ticks_per_bar = ticks_per_beat * num as u64;
                                                        let ticks_per_16th = ticks_per_beat / 4;

                                                        let bar = (total_ticks / ticks_per_bar) + 1;
                                                        let rem_bar = total_ticks % ticks_per_bar;

                                                        let beat = (rem_bar / ticks_per_beat) + 1;
                                                        let rem_beat = rem_bar % ticks_per_beat;

                                                        let sixteenth = (rem_beat / ticks_per_16th) + 1;
                                                        let tick = (rem_beat % ticks_per_16th) as u32;

                                                        let duration_ticks =
                                                                (n.duration / seconds_per_beat * ppq).round() as u64;

                                                        crate::daw::model::Note {
                                                                id: uuid::Uuid::new_v4().to_string(),
                                                                note: n.note,
                                                                start: crate::daw::model::Position {
                                                                        bar: bar as u32,
                                                                        beat: beat as u32,
                                                                        sixteenth: sixteenth as u32,
                                                                        tick,
                                                                        time: n.start,
                                                                },
                                                                duration: crate::daw::model::MusicalLength {
                                                                        bars: 0,
                                                                        beats: 0,
                                                                        sixteenths: 0,
                                                                        ticks: 0,
                                                                        total_ticks: duration_ticks,
                                                                        seconds: n.duration,
                                                                },
                                                                velocity: n.velocity,
                                                        }
                                                })
                                                .collect(),
                                        content,
                                        instrument_ids: c.instrument_ids.clone(),
                                        instrument_routes: c.instrument_routes.clone(),
                                });
                        }
                }
        }

        // Restore plugins
        {
                let mut active_plugins = state.active_plugins.lock().map_err(|_| "Lock error")?;
                active_plugins.clear();
                for p in &schema.plugins {
                        active_plugins.push(crate::daw::state::PluginInstanceData {
                                id: p.id.clone(),
                                name: p.name.clone(),
                                label: p.label.clone(),
                                routing_track_index: p.routing_track_index,
                        });
                }
        }

        // Store pending states
        {
                let mut pending = state.pending_plugin_states.lock().map_err(|_| "Lock error")?;
                *pending = plugin_states;
        }

        rebuild_engine(&state)?;

        // Apply states to new instances
        {
                let instances = state.plugin_instances.lock().map_err(|_| "Lock error")?;
                let pending = state.pending_plugin_states.lock().map_err(|_| "Lock error")?;

                for (id, instance) in instances.iter() {
                        if let Some(state_blob) = pending.get(id) {
                                if let Ok(mut inst) = instance.lock() {
                                        inst.set_state(state_blob);
                                }
                        }
                }
        }

        Ok(())
}
