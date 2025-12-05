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

#[derive(Clone, Copy)]
struct Voice {
    note: u8,
    frequency: f32,
    phase: f32,
    active: bool,
    velocity: f32,
}

impl Voice {
    fn new() -> Self {
        Self {
            note: 0,
            frequency: 0.0,
            phase: 0.0,
            active: false,
            velocity: 0.0,
        }
    }
}

pub struct WaveGenerator {
    #[allow(dead_code)]
    id: Uuid,
    voices: Vec<Voice>,
    waveform: Waveform,
}

impl WaveGenerator {
    pub fn new() -> Self {
        let mut voices = Vec::with_capacity(16);
        for _ in 0..16 {
            voices.push(Voice::new());
        }
        Self {
            id: Uuid::new_v4(),
            voices,
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

    fn get_state(&self) -> Vec<u8> {
        vec![self.waveform as u8]
    }

    fn set_state(&mut self, state: &[u8]) {
        if let Some(&val) = state.first() {
            self.waveform = Waveform::from(val as f32);
        }
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
        // 先处理事件
        for event in events {
            if let PluginEvent::Midi(midi) = event {
                match midi {
                    NoteEvent::NoteOn { note, velocity } => {
                        // Find free voice
                        let mut found = false;
                        for voice in self.voices.iter_mut() {
                            if !voice.active {
                                voice.note = *note;
                                voice.frequency = 440.0 * 2.0f32.powf((*note as f32 - 69.0) / 12.0);
                                voice.velocity = *velocity;
                                voice.active = true;
                                voice.phase = 0.0;
                                found = true;
                                println!("WaveGenerator: NoteOn {} freq {}", note, voice.frequency);
                                break;
                            }
                        }
                        if !found {
                            println!("WaveGenerator: No free voices for note {}", note);
                        }
                    }
                    NoteEvent::NoteOff { note } => {
                        for voice in self.voices.iter_mut() {
                            if voice.active && voice.note == *note {
                                voice.active = false;
                            }
                        }
                    }
                }
            } else if let PluginEvent::Parameter { id, value } = event {
                if *id == 1 {
                    self.waveform = Waveform::from(*value);
                }
            }
        }

        // Clear buffer
        buffer.samples.fill(0.0);

        let channels = buffer.channels;
        let sample_rate = buffer.sample_rate;
        let buffer_len = buffer.samples.len() / channels;

        for voice in self.voices.iter_mut() {
            if !voice.active {
                continue;
            }

            let phase_inc = voice.frequency / sample_rate;
            let amp = voice.velocity * 0.5; // Scale down to avoid clipping

            for frame_idx in 0..buffer_len {
                let sample = match self.waveform {
                    Waveform::Sine => (voice.phase * 2.0 * PI).sin(),
                    Waveform::Square => {
                        if voice.phase < 0.5 {
                            1.0
                        } else {
                            -1.0
                        }
                    }
                    Waveform::Sawtooth => 2.0 * voice.phase - 1.0,
                    Waveform::Triangle => 4.0 * (voice.phase - 0.5).abs() - 1.0,
                };

                let output = sample * amp;

                for ch in 0..channels {
                    buffer.samples[frame_idx * channels + ch] += output;
                }

                voice.phase += phase_inc;
                if voice.phase > 1.0 {
                    voice.phase -= 1.0;
                }
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
