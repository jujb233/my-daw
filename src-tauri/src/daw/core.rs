use super::state::AppState;
use crate::audio::core::plugin::Plugin;
use crate::audio::plugins::mixer::mixer_plugin::MixerPlugin;

use crate::daw::sequencer::{get_is_playing, get_playback_position};
use tauri::State;

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
    let manager = state
        .plugin_manager
        .lock()
        .map_err(|_| "Failed to lock plugin manager")?;
    for (_i, p_data) in plugins.iter().enumerate() {
        if let Some(plugin) = manager.create_plugin(&p_data.name) {
            let _inst_idx = mixer.add_instrument(plugin);
        } else if p_data.name == "SimpleSynth" {
            // Fallback for legacy
            if let Some(plugin) = manager.create_plugin("com.mydaw.simplesynth") {
                let _inst_idx = mixer.add_instrument(plugin);
            }
        }
    }

    // Load Clips into Sequencer
    let clips = state.clips.lock().map_err(|_| "Failed to lock clips")?;
    let sequencer = mixer.get_sequencer_mut();
    for clip in clips.iter() {
        sequencer.add_clip(clip.clone());
    }

    Ok(Box::new(mixer))
}

pub fn rebuild_engine(state: &State<'_, AppState>) -> Result<(), String> {
    let mut engine = state
        .audio_engine
        .lock()
        .map_err(|_| "Failed to lock audio engine")?;

    // Capture current state
    let was_running = engine.is_running();
    let is_playing = get_is_playing();
    let position = get_playback_position();

    if was_running {
        engine.stop();
    }

    // Always rebuild graph
    let root = create_audio_graph(state)?;

    // Restore state if it was running or if we want to persist state across rebuilds
    // We need to cast root back to MixerPlugin to set transport, or send an event immediately after start
    // Sending event is safer/cleaner.

    if was_running {
        engine.start(root).map_err(|e| e.to_string())?;

        // Restore transport
        use crate::audio::core::plugin::PluginEvent;
        engine.send_event(PluginEvent::Transport {
            playing: is_playing,
            position: Some(position),
            tempo: None, // TODO: Get from state
        });
    }

    Ok(())
}
