use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;
use crate::audio::core::plugin::{Plugin, AudioBuffer, PluginEvent, PluginInfo, PluginType};
use std::sync::OnceLock;

// Global storage for meter levels
pub static METER_LEVELS: OnceLock<Mutex<HashMap<Uuid, f32>>> = OnceLock::new();

pub fn get_meter_levels() -> HashMap<Uuid, f32> {
    METER_LEVELS.get_or_init(|| Mutex::new(HashMap::new()))
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
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer, _events: &[PluginEvent]) {
        let mut sum_sq = 0.0;
        let len = buffer.samples.len();
        
        if len > 0 {
            for sample in buffer.samples.iter() {
                sum_sq += sample * sample;
            }
            let rms = (sum_sq / len as f32).sqrt();
            
            // Simple smoothing
            self.current_level = self.current_level * 0.8 + rms * 0.2;
            
            // Update global map
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
        // No params
    }
}
