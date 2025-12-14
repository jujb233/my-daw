use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::Path;

use crate::daw::model::Clip;
use crate::daw::state::PluginInstanceData;

pub fn init_db(path: &Path) -> Result<Connection> {
        if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        conn.execute(
                "CREATE TABLE IF NOT EXISTS plugins (
            id TEXT PRIMARY KEY,
            name TEXT,
            state BLOB
        )",
                [],
        )?;

        conn.execute(
                "CREATE TABLE IF NOT EXISTS notes (
            clip_id TEXT,
            note_index INTEGER,
            note INTEGER,
            start REAL,
            duration REAL,
            velocity REAL,
            PRIMARY KEY (clip_id, note_index)
        )",
                [],
        )?;

        Ok(conn)
}

pub fn save_plugin_states(
        conn: &mut Connection,
        plugins: &Vec<PluginInstanceData>,
        instances: &std::collections::HashMap<
                String,
                std::sync::Arc<std::sync::Mutex<Box<dyn crate::audio::core::plugin::Plugin>>>,
        >,
) -> Result<()> {
        let tx = conn.transaction()?;
        for plugin in plugins.iter() {
                let mut state_blob = if let Some(instance) = instances.get(&plugin.id) {
                        if let Ok(inst) = instance.lock() {
                                inst.get_state()
                        } else {
                                Vec::new()
                        }
                } else {
                        Vec::new()
                };

                // 如果插件没有提供二进制 state（空），尝试通过参数集合序列化回退保存
                if state_blob.is_empty() {
                        if let Some(instance) = instances.get(&plugin.id) {
                                if let Ok(inst) = instance.lock() {
                                        let params = inst.get_parameters();
                                        if !params.is_empty() {
                                                // 构造 JSON 格式：{"__param_state":true, "params":[{"id":..,"value":..},...]}
                                                let mut jparams = Vec::new();
                                                for p in &params {
                                                        let v = inst.get_param(p.id);
                                                        jparams.push(serde_json::json!({"id": p.id, "value": v}));
                                                }
                                                let wrapper =
                                                        serde_json::json!({"__param_state": true, "params": jparams});
                                                if let Ok(s) = serde_json::to_vec(&wrapper) {
                                                        state_blob = s;
                                                }
                                        }
                                }
                        }
                }

                tx.execute(
                        "INSERT OR REPLACE INTO plugins (id, name, state) VALUES (?1, ?2, ?3)",
                        params![plugin.id, plugin.name, state_blob],
                )?;
        }
        tx.commit()?;
        Ok(())
}

pub fn save_notes(conn: &mut Connection, clips: &Vec<Clip>) -> Result<()> {
        let tx = conn.transaction()?;
        for clip in clips.iter() {
                for (idx, note) in clip.notes.iter().enumerate() {
                        tx.execute(
                "INSERT OR REPLACE INTO notes (clip_id, note_index, note, start, duration, velocity) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![clip.id, idx as i64, note.note, note.start.time, note.duration.seconds, note.velocity],
            )?;
                }
        }
        tx.commit()?;
        Ok(())
}

pub fn load_plugin_states(path: &Path) -> Result<std::collections::HashMap<String, Vec<u8>>> {
        let db_path = path.join("data.db");
        let conn = Connection::open(&db_path)?;
        let mut stmt = conn.prepare("SELECT id, state FROM plugins")?;

        let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?;
        let mut states = std::collections::HashMap::new();
        for row in rows {
                let (id, state) = row?;
                states.insert(id, state);
        }
        Ok(states)
}
