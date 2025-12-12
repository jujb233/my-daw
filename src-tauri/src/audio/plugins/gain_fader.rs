use crate::audio::core::plugin::{
    AudioBuffer, ParameterType, Plugin, PluginEvent, PluginInfo, PluginParameter, PluginType,
};
use uuid::Uuid;

pub struct GainFader {
    #[allow(dead_code)]
    id: Uuid,
    gain: f32,
}

impl GainFader {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            gain: 0.5, // 默认 -6dB
        }
    }
}

impl Plugin for GainFader {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Gain Fader".to_string(),
            vendor: "My DAW".to_string(),
            url: "".to_string(),
            plugin_type: PluginType::Native,
            unique_id: "com.mydaw.gainfader".to_string(),
        }
    }

    fn get_parameters(&self) -> Vec<PluginParameter> {
        vec![PluginParameter {
            id: 0,
            name: "Gain".to_string(),
            min_value: 0.0,
            max_value: 1.0,
            default_value: 0.5,
            value_type: ParameterType::Float,
        }]
    }

    fn get_state(&self) -> Vec<u8> {
        self.gain.to_le_bytes().to_vec()
    }

    fn set_state(&mut self, state: &[u8]) {
        if state.len() >= 4 {
            let bytes: [u8; 4] = state[0..4].try_into().unwrap();
            self.gain = f32::from_le_bytes(bytes);
        }
    }

    fn process(
        &mut self,
        buffer: &mut AudioBuffer,
        events: &[PluginEvent],
        _output_events: &mut Vec<PluginEvent>,
    ) {
        // 处理参数更新
        for event in events {
            if let PluginEvent::Parameter { id, value } = event {
                if *id == 0 {
                    // ID 0 = 增益
                    self.gain = *value;
                }
            }
        }

        // 应用增益
        for sample in buffer.samples.iter_mut() {
            *sample *= self.gain;
        }
    }

    fn get_param(&self, id: u32) -> f32 {
        if id == 0 { self.gain } else { 0.0 }
    }

    fn set_param(&mut self, id: u32, value: f32) {
        if id == 0 {
            self.gain = value;
        }
    }
}
