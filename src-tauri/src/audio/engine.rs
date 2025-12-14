use crate::audio::core::plugin::{AudioBuffer, Plugin, PluginEvent};
use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{Receiver, Sender, unbounded};

pub struct AudioEngine {
        // 当前输出流；`None` 表示未启动
        stream: Option<cpal::Stream>,
        // 发送到音频回调线程的插件事件通道
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
                let device = host
                        .default_output_device()
                        .ok_or(anyhow::anyhow!("No output device available"))?;

                let config = device.default_output_config()?;
                let sample_format = config.sample_format();
                let config: cpal::StreamConfig = config.into();
                let channels = config.channels as usize;
                let sample_rate = config.sample_rate.0 as f32;

                println!("Audio Device: {:?}", device.name());
                println!("Sample Rate: {}, Channels: {}", sample_rate, channels);

                // 创建事件通道：主线程可通过 `send_event` 发送事件到音频回调
                let (tx, rx): (Sender<PluginEvent>, Receiver<PluginEvent>) = unbounded();
                self.command_sender = Some(tx);

                let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

                let stream = match sample_format {
                        cpal::SampleFormat::F32 => device.build_output_stream(
                                &config,
                                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                                        // 非阻塞读取该音频块期间到达的所有事件
                                        let mut events = Vec::new();
                                        while let Ok(event) = rx.try_recv() {
                                                events.push(event);
                                        }

                                        let mut buffer = AudioBuffer {
                                                samples: data,
                                                channels,
                                                sample_rate,
                                        };

                                        // 插件就地处理 samples，可能产生输出事件
                                        let mut output_events = Vec::new();
                                        plugin.process(&mut buffer, &events, &mut output_events);
                                },
                                err_fn,
                                None,
                        )?,
                        _ => {
                                return Err(anyhow::anyhow!(
                                        "Unsupported sample format: {:?}",
                                        sample_format
                                ));
                        }
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
