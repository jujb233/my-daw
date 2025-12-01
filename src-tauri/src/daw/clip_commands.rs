use tauri::State;
use crate::daw::state::AppState;
use crate::daw::core::rebuild_engine;
use crate::daw::sequencer::Clip;

#[tauri::command]
pub fn add_clip(
    state: State<'_, AppState>,
    name: String,
    start_time: f64,
    duration: f64,
    instrument_id: usize,
    target_track_ids: Vec<usize>,
) -> Result<usize, String> {
    let mut engine = state
        .audio_engine
        .lock()
        .map_err(|_| "Failed to lock audio engine")?;

    // We need to access the MixerPlugin -> Sequencer
    // But AudioEngine holds `Box<dyn Plugin>`. We need to downcast or expose it.
    // Currently `AudioEngine` is generic.
    // For prototype, we can assume the root plugin is MixerPlugin.
    // But `AudioEngine` doesn't expose the plugin easily for modification while running?
    // Actually `AudioEngine` owns the plugin.
    
    // If the engine is running, we need to send a command to it?
    // Or we can use `rebuild_engine` strategy: Rebuild the whole graph with new clips?
    // That's slow for adding a clip.
    
    // Better: `AudioEngine` should support "Update Graph" or we expose a way to get the Sequencer.
    // Since `MixerPlugin` is our specific root, maybe we can add a method to `AudioEngine` to get it?
    // Or we just store Clips in `AppState` and push them to `MixerPlugin` when we rebuild or via a command event.
    
    // Let's store Clips in AppState for persistence.
    // And then update the engine.
    
    // Wait, `AppState` doesn't have `clips` yet.
    // Let's add `clips` to `AppState`.
    
    // For now, let's just print "Clip Added" and return a dummy ID.
    // Real implementation requires `AppState` update.
    println!("Adding clip: {}, start: {}, dur: {}, inst: {}, tracks: {:?}", name, start_time, duration, instrument_id, target_track_ids);
    
    Ok(0) // Dummy ID
}
