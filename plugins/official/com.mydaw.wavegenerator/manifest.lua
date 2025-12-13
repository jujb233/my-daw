return {
  id = "com.mydaw.wavegenerator",
  name = "Wave Generator",
  version = "0.1.0",
  api_version = "1",
  backend = { type = "local", path = "backend/target/release/libwave_generator_plugin.so" },
  frontend = nil,
  copy_on_project_save = true,
  capabilities = { "audio", "stereo" },
}
