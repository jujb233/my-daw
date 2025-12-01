use crate::audio::processor::{AudioProcessor, ProcessContext};
use std::f32::consts::PI;

pub struct SineWave {
    frequency: f32,
    phase: f32,
}

impl SineWave {
    pub fn new(frequency: f32) -> Self {
        Self {
            frequency,
            phase: 0.0,
        }
    }
}

impl AudioProcessor for SineWave {
    fn process(&mut self, output: &mut [f32], channels: usize, context: &ProcessContext) {
        for frame in output.chunks_mut(channels) {
            // Generate sine wave sample: sin(2 * PI * phase)
            // Amplitude is 0.1 to avoid blowing ears out during testing
            let sample = (self.phase * 2.0 * PI).sin() * 0.1;

            // Write same sample to all channels (mono -> stereo/etc)
            for channel_sample in frame.iter_mut() {
                *channel_sample = sample;
            }

            // Advance phase
            self.phase += self.frequency / context.sample_rate;
            if self.phase > 1.0 {
                self.phase -= 1.0;
            }
        }
    }
}
