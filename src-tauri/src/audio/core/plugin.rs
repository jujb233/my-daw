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
}

pub struct AudioBuffer<'a> {
    pub samples: &'a mut [f32],
    pub channels: usize,
    pub sample_rate: f32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PluginType {
    Native,
    Clap,
    Vst,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PluginInfo {
    pub name: String,
    pub vendor: String,
    pub url: String,
    pub plugin_type: PluginType,
}

pub trait Plugin: Send + Sync {
    #[allow(dead_code)]
    fn info(&self) -> PluginInfo;

    /// Process audio block.
    /// `buffer` contains the audio data to be processed (in-place).
    /// `events` contains MIDI or parameter events for this block.
    fn process(&mut self, buffer: &mut AudioBuffer, events: &[PluginEvent]);

    // Parameter handling
    #[allow(dead_code)]
    fn get_param(&self, id: u32) -> f32;
    fn set_param(&mut self, id: u32, value: f32);
}
