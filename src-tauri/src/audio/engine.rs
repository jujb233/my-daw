use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use anyhow::Result;
use crate::audio::processor::{AudioProcessor, ProcessContext};

pub struct AudioEngine {
    // The stream is kept alive here. Dropping it stops audio.
    stream: Option<cpal::Stream>,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self { stream: None }
    }

    pub fn start(&mut self, mut processor: Box<dyn AudioProcessor>) -> Result<()> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(anyhow::anyhow!("No output device available"))?;
        
        let config = device.default_output_config()?;
        let sample_format = config.sample_format();
        let config: cpal::StreamConfig = config.into();
        let channels = config.channels as usize;
        let sample_rate = config.sample_rate.0 as f32;

        println!("Audio Device: {:?}", device.name());
        println!("Sample Rate: {}, Channels: {}", sample_rate, channels);

        let context = ProcessContext { sample_rate };

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Zero out buffer first (optional, but good practice if processor adds to it)
                    // But our processor overwrites, so it's fine.
                    // Actually, let's make sure processor overwrites.
                    processor.process(data, channels, &context);
                },
                err_fn,
                None,
            )?,
            // TODO: Handle other formats by converting
            _ => return Err(anyhow::anyhow!("Unsupported sample format: {:?}", sample_format)),
        };

        stream.play()?;
        self.stream = Some(stream);
        
        Ok(())
    }
    
    pub fn stop(&mut self) {
        self.stream = None;
    }

    pub fn is_running(&self) -> bool {
        self.stream.is_some()
    }
}
