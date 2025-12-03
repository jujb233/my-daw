use crate::audio::core::plugin::{
    AudioBuffer, IOConfig, NoteEvent, ParameterType, Plugin, PluginEvent, PluginInfo,
    PluginParameter, PluginType,
};
use std::f32::consts::PI;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Sine = 0,
    Square = 1,
    Sawtooth = 2,
    Triangle = 3,
}

impl From<f32> for Waveform {
    fn from(v: f32) -> Self {
        match v as u32 {
            0 => Waveform::Sine,
            1 => Waveform::Square,
            2 => Waveform::Sawtooth,
            3 => Waveform::Triangle,
            _ => Waveform::Sine,
        }
    }
}

pub struct WaveGenerator {
    #[allow(dead_code)]
    id: Uuid,
    phase: f32,
    frequency: f32,
    active: bool,
    waveform: Waveform,
}

impl WaveGenerator {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            phase: 0.0,
            frequency: 440.0,
            active: false,
            waveform: Waveform::Sine,
        }
    }
}

impl Plugin for WaveGenerator {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Wave Generator".to_string(),
            vendor: "My DAW".to_string(),
            url: "".to_string(),
            plugin_type: PluginType::Native,
            unique_id: "com.mydaw.wavegenerator".to_string(),
        }
    }

    fn get_parameters(&self) -> Vec<PluginParameter> {
        vec![PluginParameter {
            id: 1,
            name: "Waveform".to_string(),
            min_value: 0.0,
            max_value: 3.0,
            default_value: 0.0,
            value_type: ParameterType::Enum(vec![
                "Sine".to_string(),
                "Square".to_string(),
                "Sawtooth".to_string(),
                "Triangle".to_string(),
            ]),
        }]
    }

    fn get_io_config(&self) -> IOConfig {
        IOConfig {
            inputs: 0,
            outputs: 2,
        }
    }

    fn process(
        &mut self,
        buffer: &mut AudioBuffer,
        events: &[PluginEvent],
        _output_events: &mut Vec<PluginEvent>,
    ) {
        // Handle events first
        for event in events {
            if let PluginEvent::Midi(midi) = event {
                match midi {
                    NoteEvent::NoteOn { note, .. } => {
                        // Simple MIDI to Hz conversion: 440 * 2^((note - 69) / 12)
                        self.frequency = 440.0 * 2.0f32.powf((*note as f32 - 69.0) / 12.0);
                        self.active = true;
                    }
                    NoteEvent::NoteOff { .. } => {
                        self.active = false;
                    }
                }
            } else if let PluginEvent::Parameter { id, value } = event {
                if *id == 1 {
                    self.waveform = Waveform::from(*value);
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
            let sample = match self.waveform {
                Waveform::Sine => (self.phase * 2.0 * PI).sin(),
                Waveform::Square => {
                    if self.phase < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Waveform::Sawtooth => 2.0 * self.phase - 1.0,
                Waveform::Triangle => 4.0 * (self.phase - 0.5).abs() - 1.0,
            };

            for channel_sample in frame.iter_mut() {
                *channel_sample = sample;
            }

            self.phase += self.frequency / sample_rate;
            if self.phase > 1.0 {
                self.phase -= 1.0;
            }
        }
    }

    fn get_param(&self, id: u32) -> f32 {
        if id == 1 {
            self.waveform as u32 as f32
        } else {
            0.0
        }
    }

    fn set_param(&mut self, id: u32, value: f32) {
        if id == 1 {
            self.waveform = Waveform::from(value);
        }
    }
}
