use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum NoteEvent {
    NoteOn { note: u8, velocity: f32 },
    NoteOff { note: u8 },
}

#[derive(Debug, Clone)]
pub enum PluginEvent {
    Midi(NoteEvent),
    Parameter { id: u32, value: f32 },
}

pub struct AudioBuffer<'a> {
    pub samples: &'a mut [f32],
    pub channels: usize,
    pub sample_rate: f32,
}

pub trait Plugin: Send + Sync {
    fn id(&self) -> Uuid;
    fn name(&self) -> &str;
    
    /// Process audio block.
    /// `buffer` contains the audio data to be processed (in-place).
    /// `events` contains MIDI or parameter events for this block.
    fn process(&mut self, buffer: &mut AudioBuffer, events: &[PluginEvent]);

    // Parameter handling
    fn get_param(&self, id: u32) -> f32;
    fn set_param(&mut self, id: u32, value: f32);
}
