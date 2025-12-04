use super::state::AppState;
use crate::audio::core::plugin::Plugin;
use crate::audio::plugins::mixer::mixer_plugin::MixerPlugin;

use crate::daw::sequencer::{get_is_playing, get_playback_position};
use tauri::State;

pub fn create_audio_graph(state: &State<'_, AppState>) -> Result<Box<dyn Plugin>, String> {
    let plugins = state
        .active_plugins
        .lock()
        .map_err(|_| "Failed to lock plugins list")?;

    let tracks = state
        .mixer_tracks
        .lock()
        .map_err(|_| "Failed to lock mixer tracks")?;

    let mut mixer = MixerPlugin::new(0);

    // 创建配置的轨道
    for track_data in tracks.iter() {
        mixer.add_track(track_data.meter_id);
    }

    // 添加乐器到混音台 (机架)
    let manager = state
        .plugin_manager
        .lock()
        .map_err(|_| "Failed to lock plugin manager")?;

    // 映射 UUID -> 混音台索引
    let mut inst_uuid_to_index = std::collections::HashMap::new();

    println!("Core: Building Audio Graph");
    for (_i, p_data) in plugins.iter().enumerate() {
        let inst_idx = if let Some(plugin) = manager.create_plugin(&p_data.name) {
            Some(mixer.add_instrument(plugin))
        } else if p_data.name == "SimpleSynth" {
            // 兼容旧版本的后备方案
            if let Some(plugin) = manager.create_plugin("com.mydaw.simplesynth") {
                Some(mixer.add_instrument(plugin))
            } else {
                None
            }
        } else {
            None
        };

        if let Some(idx) = inst_idx {
            println!(
                "Core: Mapped Plugin UUID {} ({}) to Index {}",
                p_data.id, p_data.name, idx
            );
            inst_uuid_to_index.insert(p_data.id.clone(), idx);
        } else {
            println!(
                "Core: Failed to create plugin for UUID {} ({})",
                p_data.id, p_data.name
            );
        }
    }

    // 加载 Clip 到音序器
    let clips = state.clips.lock().map_err(|_| "Failed to lock clips")?;
    let arrangement_tracks = state
        .arrangement_tracks
        .lock()
        .map_err(|_| "Failed to lock arrangement tracks")?;

    // 创建 ArrangementTrack ID -> Target Mixer Track ID 的映射
    let mut track_routing = std::collections::HashMap::new();
    for track in arrangement_tracks.iter() {
        track_routing.insert(track.id, track.target_mixer_track_id);
    }

    let sequencer = mixer.get_sequencer_mut();
    for clip in clips.iter() {
        println!(
            "Core: Processing Clip {}. Raw Instrument IDs: {:?}",
            clip.name, clip.instrument_ids
        );
        // 映射乐器 UUID 到索引
        let mut instrument_ids = Vec::new();
        for uuid in &clip.instrument_ids {
            if let Some(&idx) = inst_uuid_to_index.get(uuid) {
                instrument_ids.push(idx);
            } else {
                println!(
                    "Core: Warning - Clip {} references unknown instrument UUID {}",
                    clip.name, uuid
                );
            }
        }
        println!(
            "Core: Clip {} mapped to instruments {:?}",
            clip.name, instrument_ids
        );

        // 映射路由
        let mut instrument_routes = std::collections::HashMap::new();

        // 首先，填充显式路由 (Clip 级别的覆盖)
        for (uuid, track_idx) in &clip.instrument_routes {
            if let Some(&inst_idx) = inst_uuid_to_index.get(uuid) {
                instrument_routes.insert(inst_idx, vec![*track_idx]);
            }
        }

        // 然后，确保所有乐器都有路由
        // 默认路由到 Arrangement Track 指定的 Mixer Track
        let default_target_track = track_routing.get(&clip.track_id).cloned().unwrap_or(0); // 默认为 Master (0)

        for uuid in &clip.instrument_ids {
            if let Some(&inst_idx) = inst_uuid_to_index.get(uuid) {
                if !instrument_routes.contains_key(&inst_idx) {
                    instrument_routes.insert(inst_idx, vec![default_target_track]);
                }
            }
        }
        let audio_clip = crate::audio::core::clip::Clip {
            id: clip.id.clone(),
            name: clip.name.clone(),
            start_time: clip.start.time,
            duration: clip.length.seconds,
            instrument_ids,
            instrument_routes,
            notes: clip
                .notes
                .iter()
                .map(|n| crate::audio::core::clip::Note {
                    relative_start: n.start.time,
                    duration: n.duration.seconds,
                    note: n.note,
                    velocity: n.velocity,
                })
                .collect(),
        };
        sequencer.add_clip(audio_clip);
    }

    Ok(Box::new(mixer))
}

pub fn rebuild_engine(state: &State<'_, AppState>) -> Result<(), String> {
    let mut engine = state
        .audio_engine
        .lock()
        .map_err(|_| "Failed to lock audio engine")?;

    // 捕获当前状态
    let was_running = engine.is_running();
    let is_playing = get_is_playing();
    let position = get_playback_position();

    if was_running {
        engine.stop();
    }

    // 始终重建图（graph）
    let root = create_audio_graph(state)?;

    // 如果之前正在运行或我们希望在重建时保持状态，请恢复状态
    // 我们需要把 root 转回 MixerPlugin 才能设置传输状态，或者在启动后立即发送事件
    // 发送事件更安全/更清晰。

    if was_running {
        engine.start(root).map_err(|e| e.to_string())?;

        // 恢复传输状态（transport）
        use crate::audio::core::plugin::PluginEvent;
        engine.send_event(PluginEvent::Transport {
            playing: is_playing,
            position: Some(position),
            tempo: None, // TODO: 从状态获取节奏（tempo）
        });
    }

    Ok(())
}
