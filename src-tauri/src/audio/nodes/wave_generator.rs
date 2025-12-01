use uuid::Uuid;
use std::f32::consts::PI;
use crate::audio::plugin::{Plugin, AudioBuffer, PluginEvent, NoteEvent};

pub struct WaveGenerator {
    id: Uuid,
    phase: f32,
    frequency: f32,
    active: bool,
}

impl WaveGenerator {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            phase: 0.0,
            frequency: 440.0,
            active: false,
        }
    }
}

impl Plugin for WaveGenerator {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Wave Generator"
    }

    fn process(&mut self, buffer: &mut AudioBuffer, events: &[PluginEvent]) {
        // Handle events first
        for event in events {
            if let PluginEvent::Midi(midi) = event {
                match midi {
                    NoteEvent::NoteOn { note, .. } => {
                        // Simple MIDI to Hz conversion: 440 * 2^((note - 69) / 12)
                        self.frequency = 440.0 * 2.0f32.powf((*note as f32 - 69.0) / 12.0);
                        self.active = true;
                    },
                    NoteEvent::NoteOff { .. } => {
                        self.active = false;
                    }
                }
            }
        }

        if !self.active {
            // Silence
            buffer.samples.fill(0.0);
            return;
        }

        let channels = buffer.channels;
        let sample_rate = buffer.sample_rate;

        for frame in buffer.samples.chunks_mut(channels) {
            let sample = (self.phase * 2.0 * PI).sin();
            
            for channel_sample in frame.iter_mut() {
                *channel_sample = sample;
            }

            self.phase += self.frequency / sample_rate;
            if self.phase > 1.0 {
                self.phase -= 1.0;
            }
        }
    }

    fn get_param(&self, _id: u32) -> f32 {
        0.0
    }

    fn set_param(&mut self, _id: u32, _value: f32) {}
}
