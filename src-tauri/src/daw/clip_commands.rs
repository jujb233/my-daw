use crate::daw::core::rebuild_engine;
use crate::daw::model::{Clip, MusicalLength, Note, Position};
use crate::daw::state::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn add_clip(
    state: State<'_, AppState>,
    track_id: usize,
    name: String,
    start: Position,
    length: MusicalLength,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();

    {
        let mut clips = state.clips.lock().map_err(|_| "Failed to lock clips")?;
        clips.push(Clip {
            id: id.clone(),
            track_id,
            name,
            color: "#3b82f6".to_string(),
            start,
            length,
            notes: vec![],
            instrument_id: None,
        });
    }

    rebuild_engine(&state)?;
    Ok(id)
}

#[tauri::command]
pub fn update_clip(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    start: Option<Position>,
    length: Option<MusicalLength>,
    notes: Option<Vec<Note>>,
    instrument_id: Option<String>,
) -> Result<(), String> {
    let updated_clip = {
        let mut clips = state.clips.lock().map_err(|_| "Failed to lock clips")?;
        if let Some(clip) = clips.iter_mut().find(|c| c.id == id) {
            if let Some(n) = name {
                clip.name = n;
            }
            if let Some(s) = start {
                clip.start = s;
            }
            if let Some(l) = length {
                clip.length = l;
            }
            if let Some(n) = notes {
                clip.notes = n;
            }
            if let Some(i) = instrument_id {
                clip.instrument_id = Some(i);
            }
            Some(clip.clone())
        } else {
            return Err("Clip not found".to_string());
        }
    };

    if updated_clip.is_some() {
        rebuild_engine(&state)?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_clip(state: State<'_, AppState>, id: String) -> Result<Clip, String> {
    let clips = state.clips.lock().map_err(|_| "Failed to lock clips")?;
    if let Some(clip) = clips.iter().find(|c| c.id == id) {
        Ok(clip.clone())
    } else {
        Err("Clip not found".to_string())
    }
}

#[tauri::command]
pub fn remove_clip(state: State<'_, AppState>, id: String) -> Result<(), String> {
    {
        let mut clips = state.clips.lock().map_err(|_| "Failed to lock clips")?;
        if let Some(index) = clips.iter().position(|c| c.id == id) {
            clips.remove(index);
        } else {
            return Err("Clip not found".to_string());
        }
    }
    rebuild_engine(&state)?;
    Ok(())
}
