use anyhow::Result;
use std::path::Path;
pub mod db;
pub mod load;
pub mod lua;
pub mod save_plugins;
pub mod save;

use crate::daw::state::AppState;

pub struct ProjectManager;

impl ProjectManager {
        pub fn save_project(state: &AppState, project_path: &Path) -> Result<()> {
                save::save_project(state, project_path)
        }

        pub fn load_project(path: &Path) -> Result<crate::daw::serialization::schema::ProjectSchema> {
                load::load_project(path)
        }

        pub fn load_plugin_states(path: &Path) -> Result<std::collections::HashMap<String, Vec<u8>>> {
                db::load_plugin_states(path)
        }
}
