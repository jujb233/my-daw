use crate::audio::core::plugin::{
    AudioBuffer, Plugin, PluginEvent, PluginInfo, PluginParameter, PluginType,
};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;
use uuid::Uuid;

// 电平计（Meter）电平的全局存储
pub static METER_LEVELS: OnceLock<Mutex<HashMap<Uuid, f32>>> = OnceLock::new();

pub fn get_meter_levels() -> HashMap<Uuid, f32> {
    METER_LEVELS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap()
        .clone()
}

pub struct LevelMeter {
    id: Uuid,
    current_level: f32,
}

impl LevelMeter {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            current_level: 0.0,
        }
    }

    pub fn with_id(id: Uuid) -> Self {
        Self {
            id,
            current_level: 0.0,
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Plugin for LevelMeter {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Level Meter".to_string(),
            vendor: "My DAW".to_string(),
            url: "".to_string(),
            plugin_type: PluginType::Native,
            unique_id: "com.mydaw.levelmeter".to_string(),
        }
    }

    fn get_parameters(&self) -> Vec<PluginParameter> {
        Vec::new()
    }

    fn process(
        &mut self,
        buffer: &mut AudioBuffer,
        _events: &[PluginEvent],
        _output_events: &mut Vec<PluginEvent>,
    ) {
        let mut sum_sq = 0.0;
        let len = buffer.samples.len();

        if len > 0 {
            for sample in buffer.samples.iter() {
                sum_sq += sample * sample;
            }
            let rms = (sum_sq / len as f32).sqrt();

            // 简单平滑处理
            self.current_level = self.current_level * 0.8 + rms * 0.2;

            // 更新全局映射
            let map_mutex = METER_LEVELS.get_or_init(|| Mutex::new(HashMap::new()));
            if let Ok(mut map) = map_mutex.lock() {
                map.insert(self.id, self.current_level);
            }
        }
    }

    fn get_param(&self, _id: u32) -> f32 {
        0.0
    }

    fn set_param(&mut self, _id: u32, _value: f32) {
        // 无参数
    }
}
