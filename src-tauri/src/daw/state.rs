use crate::audio::engine::AudioEngine;
use crate::audio::plugins::manager::PluginManager;
use crate::daw::model::Clip;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct PluginInstanceData {
    pub id: String,
    pub name: String,
    pub label: String,
    pub routing_track_index: usize,
}

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

pub struct AppState {
    pub audio_engine: Mutex<AudioEngine>,
    pub plugin_manager: Mutex<PluginManager>,
    pub active_plugins: Mutex<Vec<PluginInstanceData>>,
    pub mixer_tracks: Mutex<Vec<MixerTrackData>>,
    pub clips: Mutex<Vec<Clip>>,
}
