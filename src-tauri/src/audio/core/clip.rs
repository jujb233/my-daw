use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    pub relative_start: f64, // 相对于片段起始的秒数
    pub duration: f64,       // 秒数
    pub note: u8,            // MIDI 音符编号
    pub velocity: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,
    pub name: String,
    pub start_time: f64, // 以秒为单位
    pub duration: f64,
    pub instrument_ids: Vec<usize>,
    // 映射 乐器 ID -> 目标轨道 ID 列表
    pub instrument_routes: HashMap<usize, Vec<usize>>,
    pub notes: Vec<Note>,
}
