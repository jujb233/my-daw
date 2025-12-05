use crate::daw::core::rebuild_engine;
use crate::daw::model::{Clip, MusicalLength, Note, Position};
use crate::AppState;
use std::collections::HashMap;
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

    // 检查是否已存在同名 Clip 以进行内容同步？
    // 目前，只是创建新的。
    // 用户需求："Clip 1, Clip 2..."。
    // 我们可以在前端或此处处理自动命名。
    // "复制 Clip 会创建一个同步的实例"。
    // 这意味着我们应该查找 'name' 是否存在。

    // 但是，add_clip 通常用于 "新建空 Clip"。
    // 如果用户复制，他们可能会调用不同的命令或我们在此处处理。
    // 让我们假设 add_clip 创建一个新的唯一 Clip。

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
            content: crate::daw::model::ClipContent::Midi,
            instrument_ids: vec![],
            instrument_routes: HashMap::new(),
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
    track_id: Option<usize>,
    length: Option<MusicalLength>,
    notes: Option<Vec<Note>>,
    instrument_ids: Option<Vec<String>>,
    instrument_routes: Option<HashMap<String, usize>>,
) -> Result<(), String> {
    println!("ClipCommand: update_clip called for id: {}", id);
    println!("ClipCommand: Inputs - name: {:?}, start: {:?}, track_id: {:?}, length: {:?}, notes_count: {:?}, instrument_ids: {:?}, routes: {:?}", 
        name, start, track_id, length, notes.as_ref().map(|n| n.len()), instrument_ids, instrument_routes);

    let mut needs_rebuild = false;

    {
        let mut clips = state.clips.lock().map_err(|_| "Failed to lock clips")?;

        // 1. 找到目标 Clip 以获取其名称（标识符）
        let target_name = if let Some(clip) = clips.iter().find(|c| c.id == id) {
            clip.name.clone()
        } else {
            return Err("Clip not found".to_string());
        };

        // 2. 确定我们是在更新 "内容" (同步) 还是 "实例" (本地)
        let mut clips_to_update = Vec::new();

        // 如果我们正在更新内容字段，我们将更新所有具有相同名称的 Clip
        let is_content_update = notes.is_some()
            || length.is_some()
            || instrument_ids.is_some()
            || instrument_routes.is_some();

        if is_content_update || name.is_some() {
            for (i, c) in clips.iter().enumerate() {
                if c.name == target_name {
                    clips_to_update.push(i);
                }
            }
        } else {
            // 仅更新实例字段（开始时间, 轨道 ID）
            if let Some(pos) = clips.iter().position(|c| c.id == id) {
                clips_to_update.push(pos);
            }
        }

        for index in clips_to_update {
            if let Some(clip) = clips.get_mut(index) {
                if let Some(n) = &name {
                    clip.name = n.clone();
                }
                // 开始时间是特定于实例的，因此仅在它是目标 Clip 时更新
                if clip.id == id {
                    if let Some(s) = &start {
                        clip.start = s.clone();
                    }
                    if let Some(tid) = track_id {
                        clip.track_id = tid;
                    }
                }

                if let Some(l) = &length {
                    clip.length = l.clone();
                }
                if let Some(n) = &notes {
                    clip.notes = n.clone();
                }
                if let Some(i) = &instrument_ids {
                    clip.instrument_ids = i.clone();
                }
                if let Some(r) = &instrument_routes {
                    clip.instrument_routes = r.clone();
                }
                needs_rebuild = true;
            }
        }
    }

    if needs_rebuild {
        rebuild_engine(&state)?;
    }

    Ok(())
}

#[tauri::command]
pub fn copy_clip(
    state: State<'_, AppState>,
    original_id: String,
    new_track_id: usize,
    new_start: Position,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();

    {
        let mut clips = state.clips.lock().map_err(|_| "Failed to lock clips")?;
        let original = clips
            .iter()
            .find(|c| c.id == original_id)
            .ok_or("Original clip not found")?
            .clone();

        clips.push(Clip {
            id: id.clone(),
            track_id: new_track_id,
            name: original.name, // 同名 -> 使用相同内容
            color: original.color,
            start: new_start,
            length: original.length,
            notes: original.notes,
            content: original.content,
            instrument_ids: original.instrument_ids,
            instrument_routes: original.instrument_routes,
        });
    }

    rebuild_engine(&state)?;
    Ok(id)
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
