use uuid::Uuid;
use crate::audio::plugin::{Plugin, AudioBuffer, PluginEvent};

pub struct GainFader {
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
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Gain Fader"
    }

    fn process(&mut self, buffer: &mut AudioBuffer, events: &[PluginEvent]) {
        // Handle parameter updates
        for event in events {
            if let PluginEvent::Parameter { id, value } = event {
                if *id == 0 { // ID 0 = Gain
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
        if id == 0 { self.gain } else { 0.0 }
    }

    fn set_param(&mut self, id: u32, value: f32) {
        if id == 0 {
            self.gain = value;
        }
    }
}
