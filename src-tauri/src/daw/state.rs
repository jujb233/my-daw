use crate::audio::core::plugin::Plugin;
use crate::audio::engine::AudioEngine;
use crate::audio::plugins::manager::PluginManager;
use crate::daw::model::{ArrangementTrack, Clip};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// 序列化到前端/磁盘的最小插件实例元数据
#[derive(Clone, Serialize, Deserialize)]
pub struct PluginInstanceData {
        pub id: String,
        pub name: String,
        pub label: String,
        pub routing_track_index: usize,
}

// 混音轨道在 UI/状态中的表示（用于显示与电平映射）
#[derive(Clone, Serialize)]
pub struct MixerTrackData {
        pub id: usize,
        pub label: String,
        pub volume: f32,
        pub pan: f32,
        pub mute: bool,
        pub solo: bool,
        pub meter_id: Option<Uuid>,
}

// 应用全局状态：持有音频引擎、插件管理器、轨道、片段与实例引用等（多线程通过 Mutex/Arc 保护）
pub struct AppState {
        pub audio_engine: Mutex<AudioEngine>,
        pub plugin_manager: Mutex<PluginManager>,
        pub active_plugins: Mutex<Vec<PluginInstanceData>>,
        pub mixer_tracks: Mutex<Vec<MixerTrackData>>,
        pub arrangement_tracks: Mutex<Vec<ArrangementTrack>>,
        pub clips: Mutex<Vec<Clip>>,
        // Plugin 实例映射：UUID -> Arc<Mutex<Box<dyn Plugin>>>
        pub plugin_instances: Mutex<HashMap<String, Arc<Mutex<Box<dyn Plugin>>>>>,
        // 未应用到实例的插件序列化状态（加载项目时暂存）
        pub pending_plugin_states: Mutex<HashMap<String, Vec<u8>>>,
}
