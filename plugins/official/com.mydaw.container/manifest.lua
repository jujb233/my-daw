return {
  id = "com.mydaw.container",
  name = "Plugin Container",
  backend = {
    -- As a local plugin, the backend dynamic library is under backend/target/... after build
    type = "local",
    path = "backend/target/debug/libcom_mydaw_container.so"
  }
}
