use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    pub relative_start: f64, // Seconds relative to clip start
    pub duration: f64,       // Seconds
    pub note: u8,            // MIDI note number
    pub velocity: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clip {
    pub id: usize,
    pub name: String,
    pub start_time: f64, // In seconds
    pub duration: f64,
    pub instrument_ids: Vec<usize>,
    // Map InstrumentID -> List of Target TrackIDs
    pub instrument_routes: HashMap<usize, Vec<usize>>,
    pub notes: Vec<Note>,
}
