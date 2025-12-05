use serde::{Deserialize, Serialize};
// use std::collections::HashMap;

/// The root structure for the project data dictionary.
/// This represents the complete state of a project for serialization.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSchema {
    pub meta: ProjectMetadata,
    pub settings: ProjectSettings,
    pub tracks: Vec<TrackSchema>,
    pub mixer: MixerSchema,
    pub plugins: Vec<PluginSchema>,
    // Clips are associated with tracks, but we might store them separately in DB or here
    // For the "Lua" part, we might list them under tracks.
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectMetadata {
    pub name: String,
    pub author: String,
    pub version: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSettings {
    pub bpm: f64,
    pub sample_rate: u32,
    pub time_signature: (u32, u32),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackSchema {
    pub id: usize,
    pub name: String,
    pub color: String,
    pub track_type: TrackType,
    pub clips: Vec<ClipSchema>,
    pub target_mixer_track_id: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TrackType {
    Audio,
    Midi,
    Group,
    Return,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClipSchema {
    pub id: String,
    pub name: String,
    pub color: String,
    pub start_position: f64, // Seconds
    pub start_bar: u32,
    pub start_beat: u32,
    pub start_sixteenth: u32,
    pub start_tick: u32,
    pub duration: f64, // Seconds
    pub duration_bars: u32,
    pub duration_beats: u32,
    pub duration_sixteenths: u32,
    pub duration_ticks: u32,
    pub duration_total_ticks: u64,
    pub offset: f64,
    pub content_type: ClipContentType,
    // For MIDI, notes might be stored in SQLite if too many, or here if few.
    // We'll define a reference or inline data.
    pub note_count: usize,
    pub notes: Vec<NoteSchema>,
    pub instrument_ids: Vec<String>,
    pub instrument_routes: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClipContentType {
    Midi,
    Audio { file_path: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoteSchema {
    pub note: u8,
    pub start: f64,
    pub duration: f64,
    pub velocity: f32,
    pub channel: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MixerSchema {
    pub tracks: Vec<MixerTrackSchema>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MixerTrackSchema {
    pub id: usize,
    pub volume: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub plugin_instances: Vec<String>, // UUIDs of plugins
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginSchema {
    pub id: String, // UUID
    pub name: String,
    pub label: String,
    pub routing_track_index: usize,
    pub format: String,             // VST3, CLAP, Internal
    pub state_blob_id: Option<i64>, // Reference to SQLite blob
}
