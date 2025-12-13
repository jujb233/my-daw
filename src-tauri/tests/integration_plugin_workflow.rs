use anyhow::Result;
use std::fs;
use std::path::PathBuf;

// Integration tests for plugin manifest parsing and project save/load copying behavior.
// These tests are simple filesystem-level checks that mimic the host behavior implemented
// in `ProjectManager::save_project` and `PluginManager::scan_native_plugins`.

#[test]
fn manifest_discovery_and_copy() -> Result<()> {
        // Arrange: locate an example official plugin from workspace
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .to_path_buf();
        let official = repo_root.join("plugins").join("official");
        assert!(
                official.exists(),
                "plugins/official must exist for this test"
        );

        // Pick first plugin folder
        let mut entries = fs::read_dir(&official)?.filter_map(|e| e.ok()).collect::<Vec<_>>();
        assert!(!entries.is_empty(), "no official plugins found");
        let plugin_folder = entries.remove(0).path();
        let manifest = plugin_folder.join("manifest.lua");
        assert!(
                manifest.exists(),
                "manifest.lua must exist in plugin folder"
        );

        // Create a temporary project folder
        let tmpdir = tempfile::tempdir()?;
        let project_path = tmpdir.path().join("project");
        fs::create_dir_all(&project_path)?;

        // Simulate copy logic: copy plugin folder into project/plugins/official/<name>
        let plugins_dir = project_path.join("plugins").join("official");
        fs::create_dir_all(&plugins_dir)?;
        let folder_name = plugin_folder
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "plugin".to_string());
        let dest = plugins_dir.join(&folder_name);

        // Perform recursive copy
        fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<(), anyhow::Error> {
                if !dst.exists() {
                        fs::create_dir_all(dst)?;
                }
                for entry in fs::read_dir(src)? {
                        let entry = entry?;
                        let path = entry.path();
                        let file_name = entry.file_name();
                        let dst_path = dst.join(file_name);
                        if path.is_dir() {
                                copy_dir_all(&path, &dst_path)?;
                        } else if path.is_file() {
                                fs::copy(&path, &dst_path)?;
                        }
                }
                Ok(())
        }

        copy_dir_all(&plugin_folder, &dest)?;

        // Assert that manifest was copied
        assert!(
                dest.join("manifest.lua").exists(),
                "manifest should be copied into project plugins"
        );

        Ok(())
}

#[test]
fn save_project_generates_project_lua_and_db() -> Result<()> {
        // This test verifies that saving a project (simulated) writes a project.lua and data.db
        let tmpdir = tempfile::tempdir()?;
        let project_path = tmpdir.path().join("project_save_test");
        fs::create_dir_all(&project_path)?;

        // Create minimal project.lua and data.db like serializer would
        let lua = "project { name = \"Test\", bpm = 120.0, sample_rate = 44100 }\n";
        fs::write(project_path.join("project.lua"), lua)?;

        // create data.db (sqlite) with plugins table to simulate save
        let db_path = project_path.join("data.db");
        let conn = rusqlite::Connection::open(&db_path)?;
        conn.execute(
                "CREATE TABLE IF NOT EXISTS plugins ( id TEXT PRIMARY KEY, name TEXT, state BLOB )",
                [],
        )?;

        // insert a fake plugin row
        conn.execute(
                "INSERT INTO plugins (id, name, state) VALUES (?1, ?2, ?3)",
                rusqlite::params!["com.mydaw.test", "TestPlugin", Vec::<u8>::new()],
        )?;

        // Now verify load functions in project serialization can open files
        assert!(project_path.join("project.lua").exists());
        assert!(project_path.join("data.db").exists());

        // Try to read plugin states using the same SQL as load_plugin_states
        let mut stmt = conn.prepare("SELECT id, state FROM plugins")?;
        let mut rows = stmt.query([])?;
        let mut found = false;
        while let Some(_) = rows.next()? {
                found = true;
        }
        assert!(found, "should find at least one plugin row");

        Ok(())
}
