#[test]
fn print_available_plugins() {
        let manager = my_daw_lib::audio::plugins::manager::PluginManager::new();
        let list = manager.get_available_plugins();
        println!("Found {} plugins", list.len());
        for p in list {
                println!("Plugin: {} ({})", p.name, p.unique_id);
        }
}
