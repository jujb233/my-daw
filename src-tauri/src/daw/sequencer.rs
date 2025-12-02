use crate::audio::core::plugin::{NoteEvent, PluginEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

// Global atomic for playback position (in samples, or f64 bits)
// Using AtomicU64 to store f64 bits for thread safety without Mutex
pub static PLAYBACK_POSITION_BITS: AtomicU64 = AtomicU64::new(0);
pub static IS_PLAYING: AtomicU64 = AtomicU64::new(0); // 0 = false, 1 = true

pub fn get_playback_position() -> f64 {
    f64::from_bits(PLAYBACK_POSITION_BITS.load(Ordering::Relaxed))
}

pub fn get_is_playing() -> bool {
    IS_PLAYING.load(Ordering::Relaxed) == 1
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    pub relative_start: f64, // Seconds relative to clip start
    pub duration: f64,       // Seconds
    pub note: u8,            // MIDI note number
    pub velocity: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clip {
    pub id: usize,
    pub name: String,
    pub start_time: f64, // In seconds
    pub duration: f64,
    pub instrument_ids: Vec<usize>,
    // Map InstrumentID -> List of Target TrackIDs
    pub instrument_routes: HashMap<usize, Vec<usize>>,
    pub notes: Vec<Note>,
}

pub struct Sequencer {
    pub clips: Vec<Clip>,
    pub sample_rate: f32,
    pub current_time: f64,
    pub tempo: f64, // BPM
    pub playing: bool,
}

impl Sequencer {
    pub fn new() -> Self {
        Self {
            clips: Vec::new(),
            sample_rate: 44100.0,
            current_time: 0.0,
            tempo: 120.0,
            playing: false,
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

    // Returns:
    // 1. Events for Instruments: HashMap<InstrumentID, Vec<PluginEvent>>
    // 2. Routing for this block: HashMap<InstrumentID, Vec<TrackID>>
    pub fn process(
        &mut self,
        samples: usize,
    ) -> (HashMap<usize, Vec<PluginEvent>>, HashMap<usize, Vec<usize>>) {
        let mut events = HashMap::new();
        let mut routing = HashMap::new();

        let duration = samples as f64 / self.sample_rate as f64;

        // Calculate Loop Length (Max clip end or 8 bars)
        let mut max_end = 0.0;
        for clip in &self.clips {
            let end = clip.start_time + clip.duration;
            if end > max_end {
                max_end = end;
            }
        }
        // Default 8 bars: 8 * 4 * (60/tempo)
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

        // Handle Loop Wrapping
        let mut looped = false;
        if self.playing && end_time >= loop_length {
            end_time = loop_length; // Clamp for this block
            looped = true;
        }

        // Find active clips
        for clip in &self.clips {
            // Check overlap
            let is_active = if self.playing {
                clip.start_time < end_time && (clip.start_time + clip.duration) > self.current_time
            } else {
                self.current_time >= clip.start_time
                    && self.current_time < (clip.start_time + clip.duration)
            };

            if is_active {
                // 1. Collect Routing
                // If multiple clips use the same instrument, we merge the target tracks
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

                // 2. Collect Events (Only if playing)
                if self.playing {
                    for &inst_id in &clip.instrument_ids {
                        let inst_events = events.entry(inst_id).or_insert(Vec::new());

                        for note in &clip.notes {
                            let note_start_abs = clip.start_time + note.relative_start;
                            let note_end_abs = note_start_abs + note.duration;

                            // Check Note On
                            if note_start_abs >= self.current_time && note_start_abs < end_time {
                                println!("Sequencer: NoteOn {} at {}", note.note, note_start_abs);
                                inst_events.push(PluginEvent::Midi(NoteEvent::NoteOn {
                                    note: note.note,
                                    velocity: note.velocity,
                                }));
                            }

                            // Check Note Off
                            if note_end_abs >= self.current_time && note_end_abs < end_time {
                                inst_events.push(PluginEvent::Midi(NoteEvent::NoteOff {
                                    note: note.note,
                                }));
                            }

                            // Edge case: If we are looping at this block, and a note is still active, kill it?
                            // Or if note_end_abs > loop_length?
                            // For now, let's just force NoteOff if we are looping and the note is playing.
                            if looped && note_start_abs < end_time && note_end_abs >= end_time {
                                inst_events.push(PluginEvent::Midi(NoteEvent::NoteOff {
                                    note: note.note,
                                }));
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

        // Update global state
        PLAYBACK_POSITION_BITS.store(self.current_time.to_bits(), Ordering::Relaxed);
        IS_PLAYING.store(if self.playing { 1 } else { 0 }, Ordering::Relaxed);

        (events, routing)
    }
}
