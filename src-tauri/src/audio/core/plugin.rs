use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub enum NoteEvent {
    NoteOn {
        #[allow(dead_code)]
        note: u8,
        #[allow(dead_code)]
        velocity: f32,
    },
    NoteOff {
        #[allow(dead_code)]
        note: u8,
    },
}

#[derive(Debug, Clone)]
pub enum PluginEvent {
    Midi(NoteEvent),
    Parameter {
        id: u32,
        value: f32,
    },
    Transport {
        playing: bool,
        position: Option<f64>,
        tempo: Option<f64>,
    },
    UpdateClip(crate::audio::core::clip::Clip),
    #[allow(dead_code)]
    Custom(String),
}

pub struct AudioBuffer<'a> {
    pub samples: &'a mut [f32],
    pub channels: usize,
    pub sample_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IOConfig {
    pub inputs: usize,  // Number of input channels
    pub outputs: usize, // Number of output channels
}

impl Default for IOConfig {
    fn default() -> Self {
        Self {
            inputs: 2,
            outputs: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginType {
    Native,
    Clap,
    Vst,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    Float,
    Int,
    Bool,
    Enum(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginParameter {
    pub id: u32,
    pub name: String,
    pub min_value: f32,
    pub max_value: f32,
    pub default_value: f32,
    pub value_type: ParameterType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PluginInfo {
    pub name: String,
    pub vendor: String,
    pub url: String,
    pub plugin_type: PluginType,
    pub unique_id: String, // Added for identification
}

pub trait Plugin: Send + Sync {
    #[allow(dead_code)]
    fn info(&self) -> PluginInfo;

    fn get_parameters(&self) -> Vec<PluginParameter>;

    #[allow(dead_code)]
    fn get_io_config(&self) -> IOConfig {
        IOConfig::default()
    }

    /// Process audio block.
    /// `buffer` contains the audio data to be processed (in-place).
    /// `events` contains MIDI or parameter events for this block.
    /// `output_events` is a buffer to push outgoing events to.
    fn process(
        &mut self,
        buffer: &mut AudioBuffer,
        events: &[PluginEvent],
        output_events: &mut Vec<PluginEvent>,
    );

    // Parameter handling
    #[allow(dead_code)]
    fn get_param(&self, id: u32) -> f32;
    fn set_param(&mut self, id: u32, value: f32);
}
