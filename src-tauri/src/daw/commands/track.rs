use crate::AppState;
use crate::daw::model::ArrangementTrack;
use tauri::State;

/// Arrangement Track 命令：查询 / 添加 / 删除。
/// 删除时会保持现有 ID 不变以避免影响现有 Clip 的引用。

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
        let mut tracks = state
                .arrangement_tracks
                .lock()
                .map_err(|_| "Failed to lock arrangement tracks")?;
        let id = tracks.len();

        // 新建轨道，默认路由到 Master (0)
        tracks.push(ArrangementTrack {
                id,
                name: format!("Track {}", id + 1),
                color: "#aec6ff".to_string(),
                muted: false,
                soloed: false,
                target_mixer_track_id: 0,
        });

        Ok(())
}

#[tauri::command]
pub fn remove_arrangement_track(state: State<'_, AppState>, id: usize) -> Result<(), String> {
        let mut tracks = state
                .arrangement_tracks
                .lock()
                .map_err(|_| "Failed to lock arrangement tracks")?;
        if let Some(index) = tracks.iter().position(|t| t.id == id) {
                tracks.remove(index);
        }
        // 保持 ID 稳定（不重新索引），避免修改任何 Clip 的引用
        Ok(())
}
