pub struct ProcessContext {
    pub sample_rate: f32,
}

/// A trait for any node in the audio graph that can generate or process audio.
pub trait AudioProcessor: Send + Sync {
    /// Process audio and fill the output buffer.
    /// The buffer is interleaved if channels > 1.
    /// e.g. [L, R, L, R, ...] for stereo.
    fn process(&mut self, output: &mut [f32], channels: usize, context: &ProcessContext);
}
