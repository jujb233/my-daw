use crate::audio::core::plugin::{AudioBuffer, Plugin, PluginEvent, PluginInfo, PluginType};
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
}impl MixerPlugin {
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

    // Removed set_routing as it is now dynamic via Sequencer

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
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer, events: &[PluginEvent]) {
        let samples_len = buffer.samples.len();
        let channels = buffer.channels;
        let sample_rate = buffer.sample_rate;

        // Resize buffers
        if self.scratch_buffer.len() != samples_len {
            self.scratch_buffer = vec![0.0; samples_len];
        }
        if self.accumulator_buffer.len() != samples_len {
            self.accumulator_buffer = vec![0.0; samples_len];
        }

        // Clear Master Output
        for sample in buffer.samples.iter_mut() {
            *sample = 0.0;
        }

        // Handle Transport Events
        for event in events {
            if let PluginEvent::Transport {
                playing,
                position,
                tempo,
            } = event
            {
                self.sequencer.set_transport(*playing, *position, *tempo);
            }
        }

        // 0. Run Sequencer to get Events and Routing for this block
        let (seq_events, routing) = self.sequencer.process(samples_len);

        let num_tracks = self.tracks.len();
        let num_instruments = self.instruments.len();

        for track_idx in 0..num_tracks {
            // 1. Clear Accumulator (Track Input)
            for x in self.accumulator_buffer.iter_mut() {
                *x = 0.0;
            }

            // 2. Sum routed instruments
            for inst_idx in 0..num_instruments {
                // Check if this instrument is routed to this track in this block
                if let Some(target_tracks) = routing.get(&inst_idx) {
                    if target_tracks.contains(&track_idx) {
                        // Filter events for this instrument
                        // Combine Sequencer events + Parameter events
                        let mut inst_events: Vec<PluginEvent> = Vec::new();

                        // Add Sequencer events (Notes)
                        if let Some(evts) = seq_events.get(&inst_idx) {
                            inst_events.extend(evts.clone());
                        }

                        // Add Parameter events
                        inst_events.extend(events.iter().filter_map(|e| {
                            match e {
                                PluginEvent::Midi(_) => None, // MIDI comes from Sequencer now (mostly)
                                PluginEvent::Transport { .. } => None,
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
                            }
                        }));

                        let inst = &mut self.instruments[inst_idx];

                        let mut inst_buffer = AudioBuffer {
                            samples: &mut self.scratch_buffer,
                            channels,
                            sample_rate,
                        };

                        inst.process(&mut inst_buffer, &inst_events);

                        // Sum to accumulator
                        for (i, sample) in self.scratch_buffer.iter().enumerate() {
                            self.accumulator_buffer[i] += sample;
                        }
                    }
                }
            }

            // 3. Process Track Effects
            let track_events: Vec<PluginEvent> = events
                .iter()
                .filter_map(|e| match e {
                    PluginEvent::Midi(_) => None,
                    PluginEvent::Transport { .. } => None,
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
                    samples: &mut self.accumulator_buffer,
                    channels,
                    sample_rate,
                };

                track.container.process(&mut track_buffer, &track_events);
            }

            // 4. Sum Track Output to Master
            for (i, sample) in self.accumulator_buffer.iter().enumerate() {
                buffer.samples[i] += sample;
            }
        }
    }

    fn get_param(&self, id: u32) -> f32 {
        // Map global mixer params to track params?
        // ID scheme: TrackIndex * 100 + ParamID
        // What about Instrument Params?
        // Let's say Instrument Params are 10000 + InstIndex * 100 + ParamID

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
