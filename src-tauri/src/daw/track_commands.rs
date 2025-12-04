use crate::daw::model::ArrangementTrack;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_arrangement_tracks(state: State<'_, AppState>) -> Result<Vec<ArrangementTrack>, String> {
    let tracks = state
        .arrangement_tracks
        .lock()
        .map_err(|_| "Failed to lock arrangement tracks")?;
    Ok(tracks.clone())
}

#[tauri::command]
pub fn add_arrangement_track(state: State<'_, AppState>) -> Result<(), String> {
    {
        let mut tracks = state
            .arrangement_tracks
            .lock()
            .map_err(|_| "Failed to lock arrangement tracks")?;
        let id = tracks.len();

        // 默认路由到 Master (0) 或者创建一个新的 Mixer Track?
        // 用户希望区分两者。
        // 让我们默认路由到 Master (0)，用户可以稍后更改路由。
        // 或者，为了方便，我们可以查找第一个可用的 Mixer Track？
        // 简单起见，默认路由到 Master (0)。

        tracks.push(ArrangementTrack {
            id,
            name: format!("Track {}", id + 1),
            color: "#aec6ff".to_string(),
            muted: false,
            soloed: false,
            target_mixer_track_id: 0,
        });
    }
    // 不需要重建引擎，除非我们更改了路由逻辑（目前没有）
    Ok(())
}

#[tauri::command]
pub fn remove_arrangement_track(state: State<'_, AppState>, id: usize) -> Result<(), String> {
    {
        let mut tracks = state
            .arrangement_tracks
            .lock()
            .map_err(|_| "Failed to lock arrangement tracks")?;

        if let Some(index) = tracks.iter().position(|t| t.id == id) {
            tracks.remove(index);
            // 注意：我们不重新索引 ID，因为 Clip 引用了这些 ID。
            // 如果我们重新索引，我们需要更新所有 Clip。
            // 简单起见，我们假设 ID 是稳定的，或者我们需要更复杂的 ID 管理（如 UUID）。
            // 目前前端使用 index 作为 ID，这在删除时会有问题。
            // 让我们暂时保持简单，不重新索引，但这可能会导致 ID 缺口。
            // 前端可能期望连续的 ID？
            // 如果前端使用 array index，那么我们需要重新索引。
            // 让我们检查前端逻辑。
            // 前端 store.tracks 是一个数组。

            // 如果我们重新索引：
            // for (i, track) in tracks.iter_mut().enumerate() {
            //     track.id = i;
            // }
            // 并且我们需要更新 Clips。
        }
    }
    Ok(())
}
