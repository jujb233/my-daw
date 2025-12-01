use crate::audio::engine::AudioEngine;
use serde::Serialize;
use std::sync::Mutex;
use uuid::Uuid;

pub struct PluginInstanceData {
    pub name: String,
    pub label: String,
    pub routing_track_index: usize,
}

#[derive(Clone, Serialize)]
pub struct MixerTrackData {
    pub id: usize,
    pub label: String,
    pub volume: f32,
    pub meter_id: Option<Uuid>,
}

pub struct AppState {
    pub audio_engine: Mutex<AudioEngine>,
    pub active_plugins: Mutex<Vec<PluginInstanceData>>,
    pub mixer_tracks: Mutex<Vec<MixerTrackData>>,
}
