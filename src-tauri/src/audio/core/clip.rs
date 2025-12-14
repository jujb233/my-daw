use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
        // 片段内的相对起始时间（秒）
        pub relative_start: f64,
        // 音符持续时长（秒）
        pub duration: f64,
        // MIDI 音符号（0-127）
        pub note: u8,
        // 力度，范围通常为 0.0 - 1.0
        pub velocity: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clip {
        pub id: String,
        pub name: String,
        // 全局起始时间（秒）
        pub start_time: f64,
        // 片段时长（秒）
        pub duration: f64,
        // 使用的乐器 ID 列表
        pub instrument_ids: Vec<usize>,
        // 乐器 ID -> 目标轨道 ID 列表（路由映射）
        pub instrument_routes: HashMap<usize, Vec<usize>>,
        // 片段内的音符事件
        pub notes: Vec<Note>,
}
