use uuid::Uuid;
use crate::audio::core::plugin::{Plugin, AudioBuffer, PluginEvent, PluginInfo, PluginType};
use crate::audio::plugins::mixer::track::MixerTrack;

pub struct MixerPlugin {
    id: Uuid,
    tracks: Vec<MixerTrack>,
    scratch_buffer: Vec<f32>,
}

impl MixerPlugin {
    pub fn new(num_tracks: usize) -> Self {
        let mut tracks = Vec::new();
        for _ in 0..num_tracks {
            tracks.push(MixerTrack::new());
        }
        
        Self {
            id: Uuid::new_v4(),
            tracks,
            scratch_buffer: Vec::new(),
        }
    }
    
    pub fn add_track(&mut self) {
        self.tracks.push(MixerTrack::new());
    }
    
    pub fn remove_track(&mut self, index: usize) {
        if index < self.tracks.len() {
            self.tracks.remove(index);
        }
    }
    
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
        // Resize scratch buffer if needed
        if self.scratch_buffer.len() != buffer.samples.len() {
            self.scratch_buffer = vec![0.0; buffer.samples.len()];
        }

        // Clear main output buffer (we will sum into it)
        // Wait, if we are the root plugin, the buffer might contain garbage or silence.
        // Usually we should overwrite it.
        // But if we are an insert, we process input.
        // The Mixer usually TAKES inputs from elsewhere.
        // For now, let's assume the Mixer IS the engine root.
        // We'll clear the output buffer first.
        buffer.samples.fill(0.0);
        
        // For this prototype, we don't have inputs routed TO the tracks yet.
        // The tracks just process silence (or whatever is in their chain).
        // If we want Instruments -> Tracks, the Instruments need to be IN the tracks or routed.
        
        // TEMPORARY: Let's assume the first track contains the SimpleSynth for now?
        // Or we need to change how we build the graph.
        
        // If we want to support "Grid" -> "Track", the Grid needs to render to a buffer, 
        // and then we pass that buffer to the Track.
        
        // Since we don't have the Grid rendering logic yet, let's just process the tracks.
        // If the tracks have generators (Synths) inside them, they will produce sound.
        
        let channels = buffer.channels;
        let sample_rate = buffer.sample_rate;

        for track in &mut self.tracks {
            // 1. Prepare input for the track.
            // Currently silence.
            self.scratch_buffer.fill(0.0);
            
            let mut track_buffer = AudioBuffer {
                samples: &mut self.scratch_buffer,
                channels,
                sample_rate,
            };
            
            // 2. Process track
            track.process(&mut track_buffer, events);
            
            // 3. Mix into main buffer
            for (out_sample, track_sample) in buffer.samples.iter_mut().zip(self.scratch_buffer.iter()) {
                *out_sample += *track_sample;
            }
        }
    }

    fn get_param(&self, id: u32) -> f32 {
        // Map global mixer params to track params?
        // ID scheme: TrackIndex * 100 + ParamID
        let track_idx = (id / 100) as usize;
        let param_id = id % 100;
        
        if let Some(track) = self.tracks.get(track_idx) {
            return track.container.get_param(param_id);
        }
        0.0
    }

    fn set_param(&mut self, id: u32, value: f32) {
        let track_idx = (id / 100) as usize;
        let param_id = id % 100;
        
        if let Some(track) = self.tracks.get_mut(track_idx) {
            track.container.set_param(param_id, value);
        }
    }
}
