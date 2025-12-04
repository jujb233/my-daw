use crate::audio::core::plugin::{
    AudioBuffer, Plugin, PluginEvent, PluginInfo, PluginParameter, PluginType,
};
use crate::audio::plugins::mixer::track::MixerTrack;
use crate::daw::sequencer::Sequencer;
use uuid::Uuid;

pub struct MixerPlugin {
    #[allow(dead_code)]
    id: Uuid,
    tracks: Vec<MixerTrack>,
    instruments: Vec<Box<dyn Plugin>>,
    sequencer: Sequencer,
    scratch_buffer: Vec<f32>,
    accumulator_buffer: Vec<f32>,
}
impl MixerPlugin {
    pub fn new(num_tracks: usize) -> Self {
        let mut tracks = Vec::new();
        for _ in 0..num_tracks {
            tracks.push(MixerTrack::new(None));
        }

        Self {
            id: Uuid::new_v4(),
            tracks,
            instruments: Vec::new(),
            sequencer: Sequencer::new(),
            scratch_buffer: Vec::new(),
            accumulator_buffer: Vec::new(),
        }
    }

    pub fn get_sequencer_mut(&mut self) -> &mut Sequencer {
        &mut self.sequencer
    }

    pub fn add_track(&mut self, meter_id: Option<Uuid>) -> Uuid {
        let track = MixerTrack::new(meter_id);
        let m_id = track.meter_id;
        self.tracks.push(track);
        m_id
    }

    pub fn add_instrument(&mut self, plugin: Box<dyn Plugin>) -> usize {
        self.instruments.push(plugin);
        self.instruments.len() - 1
    }

    // 移除了 set_routing，因为它现在通过 Sequencer 动态处理

    #[allow(dead_code)]
    pub fn get_instrument_mut(&mut self, index: usize) -> Option<&mut Box<dyn Plugin>> {
        self.instruments.get_mut(index)
    }

    #[allow(dead_code)]
    pub fn remove_track(&mut self, index: usize) {
        if index < self.tracks.len() {
            self.tracks.remove(index);
        }
    }

    #[allow(dead_code)]
    pub fn get_track_mut(&mut self, index: usize) -> Option<&mut MixerTrack> {
        self.tracks.get_mut(index)
    }
}

impl Plugin for MixerPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Mixer".to_string(),
            vendor: "My DAW".to_string(),
            url: "".to_string(),
            plugin_type: PluginType::Native,
            unique_id: "com.mydaw.mixer".to_string(),
        }
    }

    fn get_parameters(&self) -> Vec<PluginParameter> {
        // 混音台参数可以在此处公开（例如主音量）
        // 目前为空。
        Vec::new()
    }

    fn process(
        &mut self,
        buffer: &mut AudioBuffer,
        events: &[PluginEvent],
        output_events: &mut Vec<PluginEvent>,
    ) {
        let samples_len = buffer.samples.len();
        let channels = buffer.channels;
        let sample_rate = buffer.sample_rate;

        // 调整缓冲区大小
        if self.scratch_buffer.len() != samples_len {
            self.scratch_buffer = vec![0.0; samples_len];
        }
        if self.accumulator_buffer.len() != samples_len {
            self.accumulator_buffer = vec![0.0; samples_len];
        }

        // 清除主输出
        for sample in buffer.samples.iter_mut() {
            *sample = 0.0;
        }

        // 处理传输和 Clip 事件
        for event in events {
            match event {
                PluginEvent::Transport {
                    playing,
                    position,
                    tempo,
                } => {
                    self.sequencer.set_transport(*playing, *position, *tempo);
                }
                _ => {}
            }
        }

        // 0. Run Sequencer to get Events and Routing for this block
        let (seq_events, routing) = self.sequencer.process(samples_len);

        let num_tracks = self.tracks.len();
        let num_instruments = self.instruments.len();

        // 准备轨道输入缓冲区
        // 我们需要为每个轨道分配缓冲区以累加乐器输出
        // 优化：我们可以使用单个缓冲池，但目前先采用分配/重置大小的方式
        // 实际上，如果 N 是动态的，我们无法轻松地在不分配的情况下持有 N 个缓冲区。
        // 但 self.tracks 在每次图（graph）重建时是固定的。
        // 使用扁平向量：track_inputs[track_idx * samples_len ..]
        let total_track_samples = num_tracks * samples_len;
        if self.accumulator_buffer.len() < total_track_samples {
            self.accumulator_buffer = vec![0.0; total_track_samples];
        } else {
            // 清空
            for x in self.accumulator_buffer.iter_mut() {
                *x = 0.0;
            }
        }

        // 1. Process Instruments (ONCE) and mix to Track Buffers
        for inst_idx in 0..num_instruments {
            // 合并音序器事件（Sequencer）和参数事件
            let mut inst_events: Vec<PluginEvent> = Vec::new();

            // 添加音序器事件（音符）
            if let Some(evts) = seq_events.get(&inst_idx) {
                inst_events.extend(evts.clone());
            }

            // 添加参数事件
            inst_events.extend(events.iter().filter_map(|e| match e {
                PluginEvent::Midi(_) => None,
                PluginEvent::Transport { .. } => None,
                PluginEvent::Custom(_) => None,
                PluginEvent::Parameter { id, value } => {
                    if *id >= 10000 {
                        let target_inst = ((*id - 10000) / 100) as usize;
                        if target_inst == inst_idx {
                            return Some(PluginEvent::Parameter {
                                id: (*id - 10000) % 100,
                                value: *value,
                            });
                        }
                    }
                    None
                }
            }));

            let inst = &mut self.instruments[inst_idx];

            let mut inst_buffer = AudioBuffer {
                samples: &mut self.scratch_buffer,
                channels,
                sample_rate,
            };

            // 处理乐器
            inst.process(&mut inst_buffer, &inst_events, output_events);

            // 混音到目标轨道
            if let Some(target_tracks) = routing.get(&inst_idx) {
                for &track_idx in target_tracks {
                    if track_idx < num_tracks {
                        let start = track_idx * samples_len;
                        let end = start + samples_len;
                        let track_slice = &mut self.accumulator_buffer[start..end];

                        for (i, sample) in self.scratch_buffer.iter().enumerate() {
                            track_slice[i] += sample;
                        }
                    }
                }
            }
        }

        // 2. Process Tracks
        for track_idx in 0..num_tracks {
            let start = track_idx * samples_len;
            let end = start + samples_len;

            // 将累积的输入复制到临时缓冲区以进行处理
            // （通常轨道处理是就地进行，但我们需要从累加器移动数据）
            self.scratch_buffer
                .copy_from_slice(&self.accumulator_buffer[start..end]);

            let track_events: Vec<PluginEvent> = events
                .iter()
                .filter_map(|e| match e {
                    PluginEvent::Midi(_) => None,
                    PluginEvent::Transport { .. } => None,
                    PluginEvent::Custom(_) => None,
                    PluginEvent::Parameter { id, value } => {
                        if *id < 10000 {
                            let target_track = (*id / 100) as usize;
                            if target_track == track_idx {
                                return Some(PluginEvent::Parameter {
                                    id: *id % 100,
                                    value: *value,
                                });
                            }
                        }
                        None
                    }
                })
                .collect();

            if let Some(track) = self.tracks.get_mut(track_idx) {
                let mut track_buffer = AudioBuffer {
                    samples: &mut self.scratch_buffer,
                    channels,
                    sample_rate,
                };

                track.process(&mut track_buffer, &track_events, output_events);

                // 3. 累加到主输出
                for (i, sample) in self.scratch_buffer.iter().enumerate() {
                    buffer.samples[i] += sample;
                }
            }
        }
    }

    fn get_param(&self, id: u32) -> f32 {
        // 将全局混音器参数映射到轨道参数？
        // ID 方案：TrackIndex * 100 + ParamID
        // 乐器参数如何处理？
        // 假设乐器参数为 10000 + InstIndex * 100 + ParamID

        if id >= 10000 {
            let inst_idx = ((id - 10000) / 100) as usize;
            let param_id = (id - 10000) % 100;
            if let Some(inst) = self.instruments.get(inst_idx) {
                return inst.get_param(param_id);
            }
            return 0.0;
        }

        let track_idx = (id / 100) as usize;
        let param_id = id % 100;

        if let Some(track) = self.tracks.get(track_idx) {
            return track.container.get_param(param_id);
        }
        0.0
    }

    fn set_param(&mut self, id: u32, value: f32) {
        if id >= 10000 {
            let inst_idx = ((id - 10000) / 100) as usize;
            let param_id = (id - 10000) % 100;
            if let Some(inst) = self.instruments.get_mut(inst_idx) {
                inst.set_param(param_id, value);
            }
            return;
        }

        let track_idx = (id / 100) as usize;
        let param_id = id % 100;

        if let Some(track) = self.tracks.get_mut(track_idx) {
            track.container.set_param(param_id, value);
        }
    }
}
