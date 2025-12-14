use anyhow::Result;
use mlua::{Lua, Table};
use rusqlite::{Connection, params};
use std::fs;
use std::path::Path;

use crate::daw::serialization::schema::*;

pub fn load_project(path: &Path) -> Result<ProjectSchema> {
        let lua = Lua::new();
        let script_path = path.join("project.lua");
        let script_content = fs::read_to_string(script_path)?;

        let globals = lua.globals();

        lua.load(r#"
        _G.project_data = { meta = {}, tracks = {}, clips = {}, mixer = {}, plugins = {} }
        function project(t) _G.project_data.meta = t end
        function track(t) table.insert(_G.project_data.tracks, t) end
        function clip(t) table.insert(_G.project_data.clips, t) end
        function mixer_strip(t) table.insert(_G.project_data.mixer, t) end
        function plugin(t) table.insert(_G.project_data.plugins, t) end
    "#)
                .exec()
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        lua.load(&script_content)
                .exec()
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let project_data: Table = globals
                .get("project_data")
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let meta: Table = project_data.get("meta").map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let tracks_tbl: Vec<Table> = project_data.get("tracks").map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let clips_tbl: Vec<Table> = project_data.get("clips").map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let _mixer_tbl: Vec<Table> = project_data.get("mixer").map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let plugins_tbl: Vec<Table> = project_data
                .get("plugins")
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let mut schema = ProjectSchema {
                meta: ProjectMetadata {
                        name: meta
                                .get("name")
                                .map_err(|e| anyhow::anyhow!(e.to_string()))
                                .unwrap_or("Untitled".to_string()),
                        author: "Unknown".to_string(),
                        version: "0.0.1".to_string(),
                        created_at: 0,
                        updated_at: 0,
                        description: "".to_string(),
                },
                settings: ProjectSettings {
                        bpm: meta
                                .get("bpm")
                                .map_err(|e| anyhow::anyhow!(e.to_string()))
                                .unwrap_or(120.0),
                        sample_rate: meta
                                .get("sample_rate")
                                .map_err(|e| anyhow::anyhow!(e.to_string()))
                                .unwrap_or(44100),
                        time_signature: (4, 4),
                },
                tracks: vec![],
                mixer: MixerSchema { tracks: vec![] },
                plugins: vec![],
        };

        for t in tracks_tbl {
                schema.tracks.push(TrackSchema {
                        id: t.get("id").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                        name: t.get("name").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                        color: t.get("color").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                        track_type: TrackType::Audio,
                        clips: vec![],
                        target_mixer_track_id: t.get("target_mixer").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                });
        }

        for plugin in plugins_tbl {
                schema.plugins.push(PluginSchema {
                        id: plugin.get("id").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                        name: plugin.get("name").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                        label: plugin
                                .get("label")
                                .unwrap_or_else(|_| plugin.get("name").unwrap_or("Unknown".to_string())),
                        routing_track_index: plugin.get("routing_track").unwrap_or(0),
                        format: plugin.get("format").unwrap_or("Internal".to_string()),
                        state_blob_id: None,
                });
        }

        // load notes from data.db
        let db_path = path.join("data.db");
        let conn = Connection::open(&db_path)?;
        let mut stmt = conn.prepare("SELECT note, start, duration, velocity FROM notes WHERE clip_id = ?1")?;

        for c in clips_tbl {
                let track_id: usize = c.get("track_id").map_err(|e| anyhow::anyhow!(e.to_string()))?;
                let clip_id: String = c.get("id").map_err(|e| anyhow::anyhow!(e.to_string()))?;

                let note_iter = stmt.query_map(params![clip_id], |row| {
                        Ok(NoteSchema {
                                note: row.get(0)?,
                                start: row.get(1)?,
                                duration: row.get(2)?,
                                velocity: row.get(3)?,
                                channel: 0,
                        })
                })?;

                let mut notes = Vec::new();
                for note in note_iter {
                        notes.push(note?);
                }

                let type_str: String = c.get("type").unwrap_or("midi".to_string());
                let audio_path: String = c.get("audio_path").unwrap_or("".to_string());

                let content_type = if type_str == "audio" {
                        let abs_path = path.join(&audio_path);
                        ClipContentType::Audio {
                                file_path: abs_path.to_string_lossy().to_string(),
                        }
                } else {
                        ClipContentType::Midi
                };

                let inst_ids_tbl: Option<Table> = c.get("instrument_ids").ok();
                let mut instrument_ids = Vec::new();
                if let Some(tbl) = inst_ids_tbl {
                        for pair in tbl.pairs::<usize, String>() {
                                if let Ok((_, id)) = pair {
                                        instrument_ids.push(id);
                                }
                        }
                }

                let inst_routes_tbl: Option<Table> = c.get("instrument_routes").ok();
                let mut instrument_routes = std::collections::HashMap::new();
                if let Some(tbl) = inst_routes_tbl {
                        for pair in tbl.pairs::<String, usize>() {
                                if let Ok((k, v)) = pair {
                                        instrument_routes.insert(k, v);
                                }
                        }
                }

                if let Some(track) = schema.tracks.iter_mut().find(|t| t.id == track_id) {
                        track.clips.push(ClipSchema {
                                id: clip_id,
                                name: c.get("name").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                                color: c.get("color").unwrap_or("#3b82f6".to_string()),
                                start_position: c.get("start").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                                start_bar: c.get("start_bar").unwrap_or(0),
                                start_beat: c.get("start_beat").unwrap_or(0),
                                start_sixteenth: c.get("start_sixteenth").unwrap_or(0),
                                start_tick: c.get("start_tick").unwrap_or(0),
                                duration: c.get("duration").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                                duration_bars: c.get("duration_bars").unwrap_or(0),
                                duration_beats: c.get("duration_beats").unwrap_or(0),
                                duration_sixteenths: c.get("duration_sixteenths").unwrap_or(0),
                                duration_ticks: c.get("duration_ticks").unwrap_or(0),
                                duration_total_ticks: c.get("duration_total_ticks").unwrap_or(0),
                                offset: 0.0,
                                content_type,
                                note_count: notes.len(),
                                notes,
                                instrument_ids,
                                instrument_routes,
                        });
                }
        }

        Ok(schema)
}
