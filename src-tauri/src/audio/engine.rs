use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use anyhow::Result;
use crossbeam_channel::{unbounded, Sender, Receiver};
use crate::audio::plugin::{Plugin, AudioBuffer, PluginEvent};

pub struct AudioEngine {
    stream: Option<cpal::Stream>,
    command_sender: Option<Sender<PluginEvent>>,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self { 
            stream: None,
            command_sender: None,
        }
    }

    pub fn start(&mut self, mut plugin: Box<dyn Plugin>) -> Result<()> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(anyhow::anyhow!("No output device available"))?;
        
        let config = device.default_output_config()?;
        let sample_format = config.sample_format();
        let config: cpal::StreamConfig = config.into();
        let channels = config.channels as usize;
        let sample_rate = config.sample_rate.0 as f32;

        println!("Audio Device: {:?}", device.name());
        println!("Sample Rate: {}, Channels: {}", sample_rate, channels);

        let (tx, rx): (Sender<PluginEvent>, Receiver<PluginEvent>) = unbounded();
        self.command_sender = Some(tx);

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Collect events from the queue
                    let mut events = Vec::new();
                    while let Ok(event) = rx.try_recv() {
                        events.push(event);
                    }

                    let mut buffer = AudioBuffer {
                        samples: data,
                        channels,
                        sample_rate,
                    };

                    plugin.process(&mut buffer, &events);
                },
                err_fn,
                None,
            )?,
            _ => return Err(anyhow::anyhow!("Unsupported sample format: {:?}", sample_format)),
        };

        stream.play()?;
        self.stream = Some(stream);
        
        Ok(())
    }
    
    pub fn stop(&mut self) {
        self.stream = None;
        self.command_sender = None;
    }

    pub fn is_running(&self) -> bool {
        self.stream.is_some()
    }

    pub fn send_event(&self, event: PluginEvent) {
        if let Some(sender) = &self.command_sender {
            let _ = sender.send(event);
        }
    }
}
