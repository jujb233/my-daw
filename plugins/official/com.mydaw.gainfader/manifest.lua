return {
  id = "com.mydaw.gainfader",
  name = "Gain Fader",
  version = "0.1.0",
  api_version = "1",
  backend = { type = "local", path = "backend/target/release/libgain_fader_plugin.so" },
  frontend = nil,
  copy_on_project_save = true,
  capabilities = { "audio", "stereo", "parameter-automation" },
}
