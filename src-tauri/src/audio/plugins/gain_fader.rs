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
            gain: 0.5, // Default -6dB
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

    fn process(
        &mut self,
        buffer: &mut AudioBuffer,
        events: &[PluginEvent],
        _output_events: &mut Vec<PluginEvent>,
    ) {
        // Handle parameter updates
        for event in events {
            if let PluginEvent::Parameter { id, value } = event {
                if *id == 0 {
                    // ID 0 = Gain
                    self.gain = *value;
                }
            }
        }

        // Apply gain
        for sample in buffer.samples.iter_mut() {
            *sample *= self.gain;
        }
    }

    fn get_param(&self, id: u32) -> f32 {
        if id == 0 {
            self.gain
        } else {
            0.0
        }
    }

    fn set_param(&mut self, id: u32, value: f32) {
        if id == 0 {
            self.gain = value;
        }
    }
}
