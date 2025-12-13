return {
  id = "com.mydaw.simplesynth",
  name = "Simple Synth",
  version = "0.1.0",
  api_version = "1",
  backend = { type = "local", path = "backend/target/release/libsimple_synth_plugin.so" },
  frontend = nil,
  copy_on_project_save = true,
  capabilities = { "midi", "stereo", "parameter-automation" },
}
