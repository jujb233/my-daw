use serde::{Deserialize, Serialize};

// DAW 数据模型：位置、时长、音符、片段（Clip）、编排轨道等
#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct TimeSignature {
        pub numerator: u32,
        pub denominator: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
        pub bar: u32,
        pub beat: u32,
        pub sixteenth: u32,
        pub tick: u32,
        pub time: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicalLength {
        pub bars: u32,
        pub beats: u32,
        pub sixteenths: u32,
        pub ticks: u32,
        pub total_ticks: u64,
        pub seconds: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
        pub id: String,
        pub note: u8,
        pub start: Position,
        pub duration: MusicalLength,
        pub velocity: f32,
}

use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClipContent {
        Midi,
        Audio { path: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
        pub id: String,
        // 引用编排轨道 ID
        pub track_id: usize,
        pub name: String,
        pub color: String,
        pub start: Position,
        pub length: MusicalLength,
        pub notes: Vec<Note>,
        pub content: ClipContent,
        // 使用的乐器 UUID 列表
        pub instrument_ids: Vec<String>,
        // 乐器 UUID -> MixerTrackID 的直接路由覆盖
        pub instrument_routes: HashMap<String, usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArrangementTrack {
        pub id: usize,
        pub name: String,
        pub color: String,
        pub muted: bool,
        pub soloed: bool,
        pub target_mixer_track_id: usize,
}
