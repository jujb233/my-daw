use std::collections::HashMap;
use crate::audio::core::plugin::{NoteEvent, PluginEvent};

#[derive(Clone, Debug)]
pub struct Clip {
    pub id: usize,
    pub name: String,
    pub start_time: f64, // In seconds (or beats? Let's use seconds for engine simplicity for now, or samples)
    pub duration: f64,
    pub instrument_id: usize,
    pub target_track_ids: Vec<usize>,
    pub notes: Vec<NoteEvent>, // Simplified for now, usually needs timestamp relative to clip start
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
    pub fn process(&mut self, samples: usize) -> (HashMap<usize, Vec<PluginEvent>>, HashMap<usize, Vec<usize>>) {
        let mut events = HashMap::new();
        let mut routing = HashMap::new();
        
        let duration = samples as f64 / self.sample_rate as f64;
        let end_time = if self.playing {
            self.current_time + duration
        } else {
            self.current_time
        };

        // Find active clips
        for clip in &self.clips {
            // Check overlap
            let is_active = if self.playing {
                clip.start_time < end_time && (clip.start_time + clip.duration) > self.current_time
            } else {
                self.current_time >= clip.start_time && self.current_time < (clip.start_time + clip.duration)
            };

            if is_active {
                // 1. Collect Routing
                // If multiple clips use the same instrument, we merge the target tracks
                let tracks = routing.entry(clip.instrument_id).or_insert(Vec::new());
                for &t_id in &clip.target_track_ids {
                    if !tracks.contains(&t_id) {
                        tracks.push(t_id);
                    }
                }

                // 2. Collect Events (Only if playing)
                if self.playing {
                    // Placeholder: If we just entered the clip, send NoteOn.
                    if clip.start_time >= self.current_time && clip.start_time < end_time {
                         let inst_events = events.entry(clip.instrument_id).or_insert(Vec::new());
                         inst_events.push(PluginEvent::Midi(NoteEvent::NoteOn { note: 60, velocity: 0.8 }));
                    }
                    
                    // If we are exiting the clip, send NoteOff?
                    if (clip.start_time + clip.duration) >= self.current_time && (clip.start_time + clip.duration) < end_time {
                         let inst_events = events.entry(clip.instrument_id).or_insert(Vec::new());
                         inst_events.push(PluginEvent::Midi(NoteEvent::NoteOff { note: 60 }));
                    }
                }
            }
        }

        if self.playing {
            self.current_time = end_time;
        }
        (events, routing)
    }
}
