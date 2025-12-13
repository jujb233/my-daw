return {
  id = "com.mydaw.levelmeter",
  name = "Level Meter",
  version = "0.1.0",
  api_version = "1",
  backend = { type = "local", path = "backend/target/release/liblevel_meter_plugin.so" },
  frontend = nil,
  copy_on_project_save = true,
  capabilities = { "meter", "stereo" },
}
