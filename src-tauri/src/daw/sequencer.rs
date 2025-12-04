use crate::audio::core::clip::Clip;
use crate::audio::core::plugin::{NoteEvent, PluginEvent};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

// 全局原子变量用于播放位置（以样本或 f64 位表示）
// 使用 AtomicU64 存储 f64 位，以便在没有 Mutex 的情况下实现线程安全
pub static PLAYBACK_POSITION_BITS: AtomicU64 = AtomicU64::new(0);
pub static IS_PLAYING: AtomicU64 = AtomicU64::new(0); // 0 = false, 1 = true

pub fn get_playback_position() -> f64 {
    f64::from_bits(PLAYBACK_POSITION_BITS.load(Ordering::Relaxed))
}

pub fn get_is_playing() -> bool {
    IS_PLAYING.load(Ordering::Relaxed) == 1
}

pub struct Sequencer {
    pub clips: Vec<Clip>,
    pub sample_rate: f32,
    pub current_time: f64,
    pub tempo: f64, // BPM（每分钟拍数）
    pub playing: bool,
    // 跟踪活动音符: InstrumentID -> Map<Note, Velocity>
    // 我们使用 Map 来处理同一音符的多个实例（如果需要），但 Set 通常就足够了。
    // 让我们只存储活动音符以便在停止时发送 NoteOff。
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

    // 返回:
    // 1. 乐器事件: HashMap<InstrumentID, Vec<PluginEvent>>
    // 2. 当前块的路由: HashMap<InstrumentID, Vec<TrackID>>
    pub fn process(
        &mut self,
        samples: usize,
    ) -> (HashMap<usize, Vec<PluginEvent>>, HashMap<usize, Vec<usize>>) {
        let mut events = HashMap::new();
        let mut routing = HashMap::new();

        let duration = samples as f64 / self.sample_rate as f64;

        // 计算循环长度（最大 Clip 结束时间或 8 小节）
        let mut max_end = 0.0;
        for clip in &self.clips {
            let end = clip.start_time + clip.duration;
            if end > max_end {
                max_end = end;
            }
        }
        // 默认 8 小节: 8 * 4 * (60/tempo)
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
            end_time = loop_length; // 钳制到当前块
            looped = true;
        }

        // 处理停止/暂停：终止所有活动音符
        if !self.playing {
            for (inst_id, notes) in self.active_notes.drain() {
                let inst_events = events.entry(inst_id).or_insert(Vec::new());
                for note in notes {
                    inst_events.push(PluginEvent::Midi(NoteEvent::NoteOff { note }));
                }
            }
        }

        // 查找活动 Clip
        for clip in &self.clips {
            // 检查重叠
            let is_active = if self.playing {
                clip.start_time < end_time && (clip.start_time + clip.duration) > self.current_time
            } else {
                self.current_time >= clip.start_time
                    && self.current_time < (clip.start_time + clip.duration)
            };

            if is_active {
                // 1. 收集路由
                // 如果多个 Clip 使用相同的乐器，我们合并目标轨道
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

                // 2. 收集事件（仅当播放时）
                if self.playing {
                    for &inst_id in &clip.instrument_ids {
                        let inst_events = events.entry(inst_id).or_insert(Vec::new());
                        let active_list = self.active_notes.entry(inst_id).or_insert(Vec::new());

                        for note in &clip.notes {
                            let note_start_abs = clip.start_time + note.relative_start;
                            let note_end_abs = note_start_abs + note.duration;

                            // 检查 Note On
                            if note_start_abs >= self.current_time && note_start_abs < end_time {
                                inst_events.push(PluginEvent::Midi(NoteEvent::NoteOn {
                                    note: note.note,
                                    velocity: note.velocity,
                                }));
                                active_list.push(note.note);
                            }

                            // 检查 Note Off
                            if note_end_abs >= self.current_time && note_end_abs < end_time {
                                inst_events.push(PluginEvent::Midi(NoteEvent::NoteOff {
                                    note: note.note,
                                }));
                                if let Some(pos) = active_list.iter().position(|&n| n == note.note)
                                {
                                    active_list.remove(pos);
                                }
                            }

                            // 边缘情况：如果我们在该块循环，并且音符仍然处于活动状态，是否将其终止？
                            // 或者如果 note_end_abs > loop_length？
                            // 目前，如果我们正在循环且音符正在播放，我们强制发送 NoteOff。
                            if looped && note_start_abs < end_time && note_end_abs >= end_time {
                                inst_events.push(PluginEvent::Midi(NoteEvent::NoteOff {
                                    note: note.note,
                                }));
                                if let Some(pos) = active_list.iter().position(|&n| n == note.note)
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
                self.current_time = 0.0;
            } else {
                self.current_time = end_time;
            }
        }

        // 更新全局状态
        PLAYBACK_POSITION_BITS.store(self.current_time.to_bits(), Ordering::Relaxed);
        IS_PLAYING.store(if self.playing { 1 } else { 0 }, Ordering::Relaxed);

        (events, routing)
    }
}
