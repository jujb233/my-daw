use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::daw::state::AppState;

use super::db as db_helpers;
use super::lua::generate_lua_script;
use super::save_plugins as plugin_helpers;

pub fn save_project(state: &AppState, project_path: &Path) -> Result<()> {
        if !project_path.exists() {
                fs::create_dir_all(project_path)?;
        }

        let db_path = project_path.join("data.db");
        let mut conn = db_helpers::init_db(&db_path)?;

        let tracks = state.arrangement_tracks.lock().unwrap();
        let clips = state.clips.lock().unwrap();
        let mixer_tracks = state.mixer_tracks.lock().unwrap();
        let plugins = state.active_plugins.lock().unwrap();

        let instances = state.plugin_instances.lock().unwrap();
        db_helpers::save_plugin_states(&mut conn, &plugins, &*instances)?;
        db_helpers::save_notes(&mut conn, &clips)?;

        plugin_helpers::copy_plugins_into_project(state, project_path);

        let lua_script = generate_lua_script(&tracks, &clips, &mixer_tracks, &plugins, project_path);
        fs::write(project_path.join("project.lua"), lua_script)?;

        Ok(())
}
