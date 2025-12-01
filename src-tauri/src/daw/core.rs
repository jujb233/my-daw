use tauri::State;
use crate::audio::core::plugin::Plugin;
use crate::audio::plugins::mixer::mixer_plugin::MixerPlugin;
use crate::audio::plugins::simple_synth::create_simple_synth;
use super::state::AppState;

pub fn create_audio_graph(state: &State<'_, AppState>) -> Result<Box<dyn Plugin>, String> {
    let plugins = state
        .active_plugins
        .lock()
        .map_err(|_| "Failed to lock plugins list")?;

    let tracks = state
        .mixer_tracks
        .lock()
        .map_err(|_| "Failed to lock mixer tracks")?;

    let mut mixer = MixerPlugin::new(0);

    // Create configured tracks
    for track_data in tracks.iter() {
        mixer.add_track(track_data.meter_id);
    }

    // Add Instruments to Mixer (Rack)
    for (i, p_data) in plugins.iter().enumerate() {
        if p_data.name == "SimpleSynth" {
            let synth = create_simple_synth();
            let inst_idx = mixer.add_instrument(synth);

            // Routing
            mixer.set_routing(inst_idx, p_data.routing_track_index);
        }
    }

    Ok(Box::new(mixer))
}

pub fn rebuild_engine(state: &State<'_, AppState>) -> Result<(), String> {
    let mut engine = state
        .audio_engine
        .lock()
        .map_err(|_| "Failed to lock audio engine")?;

    if engine.is_running() {
        engine.stop();
        let root = create_audio_graph(state)?;
        engine.start(root).map_err(|e| e.to_string())?;
    }
    Ok(())
}
