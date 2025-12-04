use serde::{Deserialize, Serialize};

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
    pub time: f64, // Seconds
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: String,
    pub track_id: usize,
    pub name: String,
    pub color: String,
    pub start: Position,
    pub length: MusicalLength,
    pub notes: Vec<Note>,
    pub instrument_id: Option<String>,
}
