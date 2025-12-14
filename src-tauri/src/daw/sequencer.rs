use crate::audio::core::clip::Clip;
use crate::audio::core::plugin::{NoteEvent, PluginEvent};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

// 全局原子量：以 f64 的位模式存储播放位置，避免在回调中使用 Mutex
pub static PLAYBACK_POSITION_BITS: AtomicU64 = AtomicU64::new(0);
pub static IS_PLAYING: AtomicU64 = AtomicU64::new(0); // 0 = false, 1 = true

pub fn get_playback_position() -> f64 {
        f64::from_bits(PLAYBACK_POSITION_BITS.load(Ordering::Relaxed))
}
pub fn get_is_playing() -> bool {
        IS_PLAYING.load(Ordering::Relaxed) == 1
}

// 简单的音序器：维护 Clips、播放时间与活动音符，并按块生成插件事件与路由映射
pub struct Sequencer {
        pub clips: Vec<Clip>,
        pub sample_rate: f32,
        pub current_time: f64,
        pub tempo: f64,
        pub playing: bool,
        pub active_notes: HashMap<usize, Vec<u8>>,
}

impl Sequencer {
        pub fn new() -> Self {
                Self {
                        clips: Vec::new(),
                        sample_rate: 44100.0,
                        current_time: 0.0,
                        tempo: 120.0,
                        playing: false,
                        active_notes: HashMap::new(),
                }
        }

        // 设置传输状态（播放/位置/节拍）
        pub fn set_transport(&mut self, playing: bool, position: Option<f64>, tempo: Option<f64>) {
                self.playing = playing;
                if let Some(pos) = position {
                        self.current_time = pos;
                }
                if let Some(t) = tempo {
                        self.tempo = t;
                }
        }

        pub fn add_clip(&mut self, clip: Clip) {
                self.clips.push(clip);
        }

        // 处理音频块并返回：
        // 1) 每个乐器的事件列表 (NoteOn/NoteOff 等)
        // 2) 每个乐器当前对应的目标轨道路由
        pub fn process(&mut self, samples: usize) -> (HashMap<usize, Vec<PluginEvent>>, HashMap<usize, Vec<usize>>) {
                let mut events = HashMap::new();
                let mut routing = HashMap::new();

                let duration = samples as f64 / self.sample_rate as f64;

                // 计算循环长度（取所有 Clip 的最大结束时间或至少 8 小节）
                let mut max_end = 0.0;
                for clip in &self.clips {
                        let end = clip.start_time + clip.duration;
                        if end > max_end {
                                max_end = end;
                        }
                }
                let min_length = 8.0 * 4.0 * (60.0 / self.tempo);
                let loop_length = if max_end > min_length {
                        max_end
                } else {
                        min_length
                };

                let mut end_time = if self.playing {
                        self.current_time + duration
                } else {
                        self.current_time
                };

                // 处理循环回绕
                let mut looped = false;
                if self.playing && end_time >= loop_length {
                        end_time = loop_length;
                        looped = true;
                }

                // 非播放状态：发送所有活动音符的 NoteOff
                if !self.playing {
                        for (inst_id, notes) in self.active_notes.drain() {
                                let inst_events = events.entry(inst_id).or_insert(Vec::new());
                                for note in notes {
                                        inst_events.push(PluginEvent::Midi(NoteEvent::NoteOff { note }));
                                }
                        }
                }

                // 遍历 Clips，收集路由与事件（仅在播放时生成 NoteOn/NoteOff）
                for clip in &self.clips {
                        let is_active = if self.playing {
                                clip.start_time < end_time && (clip.start_time + clip.duration) > self.current_time
                        } else {
                                self.current_time >= clip.start_time
                                        && self.current_time < (clip.start_time + clip.duration)
                        };

                        if is_active {
                                // 收集路由覆盖
                                for &inst_id in &clip.instrument_ids {
                                        if let Some(target_tracks) = clip.instrument_routes.get(&inst_id) {
                                                let tracks = routing.entry(inst_id).or_insert(Vec::new());
                                                for &t_id in target_tracks {
                                                        if !tracks.contains(&t_id) {
                                                                tracks.push(t_id);
                                                        }
                                                }
                                        }
                                }

                                if self.playing {
                                        for &inst_id in &clip.instrument_ids {
                                                let inst_events = events.entry(inst_id).or_insert(Vec::new());
                                                let active_list =
                                                        self.active_notes.entry(inst_id).or_insert(Vec::new());

                                                for note in &clip.notes {
                                                        let note_start_abs = clip.start_time + note.relative_start;
                                                        let note_end_abs = note_start_abs + note.duration;

                                                        // NoteOn
                                                        if note_start_abs >= self.current_time
                                                                && note_start_abs < end_time
                                                        {
                                                                inst_events.push(PluginEvent::Midi(
                                                                        NoteEvent::NoteOn {
                                                                                note: note.note,
                                                                                velocity: note.velocity,
                                                                        },
                                                                ));
                                                                active_list.push(note.note);
                                                        }

                                                        // NoteOff
                                                        if note_end_abs >= self.current_time && note_end_abs < end_time
                                                        {
                                                                inst_events.push(PluginEvent::Midi(
                                                                        NoteEvent::NoteOff { note: note.note },
                                                                ));
                                                                if let Some(pos) =
                                                                        active_list.iter().position(|&n| n == note.note)
                                                                {
                                                                        active_list.remove(pos);
                                                                }
                                                        }

                                                        // 循环边界处强制终止仍在播放的音符
                                                        if looped
                                                                && note_start_abs < end_time
                                                                && note_end_abs >= end_time
                                                        {
                                                                inst_events.push(PluginEvent::Midi(
                                                                        NoteEvent::NoteOff { note: note.note },
                                                                ));
                                                                if let Some(pos) =
                                                                        active_list.iter().position(|&n| n == note.note)
                                                                {
                                                                        active_list.remove(pos);
                                                                }
                                                        }
                                                }
                                        }
                                }
                        }
                }

                if self.playing {
                        if looped {
                                self.current_time = 0.0
                        } else {
                                self.current_time = end_time
                        }
                }

                // 更新全局播放状态
                PLAYBACK_POSITION_BITS.store(self.current_time.to_bits(), Ordering::Relaxed);
                IS_PLAYING.store(if self.playing { 1 } else { 0 }, Ordering::Relaxed);

                (events, routing)
        }
}
