return {
    id = "com.mydaw.simplesynth",
    name = "Simple Synth",
    version = "0.1.0",
    description = "A simple built-in-style synth exposed as a native plugin",
    backend = {
        type = "local",
        -- path is relative to the plugin folder; adjust if you build to a different profile
        path = "backend/target/release/libsimple_synth_plugin.so",
    },
    -- copy plugin into project when saving a project that uses it
    copy_on_project_save = true,
}
