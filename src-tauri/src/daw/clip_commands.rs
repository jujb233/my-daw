use crate::audio::core::plugin::NoteEvent;
use crate::daw::core::rebuild_engine;
use crate::daw::sequencer::Clip;
use crate::daw::state::AppState;
use tauri::State;

#[tauri::command]
pub fn add_clip(
    state: State<'_, AppState>,
    name: String,
    start_time: f64,
    duration: f64,
    instrument_id: usize,
    target_track_ids: Vec<usize>,
) -> Result<usize, String> {
    let id = {
        let mut clips = state.clips.lock().map_err(|_| "Failed to lock clips")?;
        let id = clips.len();
        clips.push(Clip {
            id,
            name,
            start_time,
            duration,
            instrument_id,
            target_track_ids,
            notes: vec![NoteEvent::NoteOn {
                note: 60,
                velocity: 0.8,
            }], // Default C4
        });
        id
    };

    rebuild_engine(&state)?;
    Ok(id)
}

#[tauri::command]
pub fn update_clip(
    state: State<'_, AppState>,
    id: usize,
    name: Option<String>,
    start_time: Option<f64>,
    duration: Option<f64>,
    instrument_id: Option<usize>,
    target_track_ids: Option<Vec<usize>>,
) -> Result<(), String> {
    {
        let mut clips = state.clips.lock().map_err(|_| "Failed to lock clips")?;
        if let Some(clip) = clips.iter_mut().find(|c| c.id == id) {
            if let Some(n) = name { clip.name = n; }
            if let Some(s) = start_time { clip.start_time = s; }
            if let Some(d) = duration { clip.duration = d; }
            if let Some(i) = instrument_id { clip.instrument_id = i; }
            if let Some(t) = target_track_ids { clip.target_track_ids = t; }
        } else {
            return Err("Clip not found".to_string());
        }
    }
    rebuild_engine(&state)?;
    Ok(())
}
