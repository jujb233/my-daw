Plugin manifest (manifest.lua) — 规范

目的

- 描述原生插件的元信息、后端与可选前端路径、复制策略和子插件结构，便于宿主自动发现、加载与在工程保存时复现。

基本要求（Lua 表，返回一个 table）

- id: string — 唯一标识，例如 "com.mydaw.gainfader"（必需）
- name: string — 显示名（必需）
- version: string — 语义化版本，例 "0.1.0"（可选但建议）
- backend: table — 后端描述（必需）
     - type: string — "local" | "builtin" | "clap"（必需）
     - path: string — 相对于 manifest 的路径，指向动态库或构建产物（当 type == "local" 时必需）
     - module: string — 内置模块名（当 type == "builtin" 时）
- frontend: table — 前端资源（可选）
     - path: string — 相对于 manifest 的前端目录或捆绑文件
- copy_on_project_save: boolean — 是否在工程保存时复制到工程目录；默认 true（可选）
- children: table[] — 子插件/依赖插件的列表（可选）

children 表示（示例）
children = {
{
id = "com.mydaw.somechild",
backend = { type = "local", path = "../child_plugin/backend/target/release/libchild.so" }
}
}

行为约定

- 宿主扫描 plugin 目录（例如 `plugins/official` 或 `project/plugins`），找到包含 `manifest.lua` 的子目录并解析它。
- 当 `backend.type == "local"` 时，`backend.path` 相对于 manifest 所在目录解析为实际文件路径。
- 在保存工程时：宿主会根据 `copy_on_project_save` 字段决定是否把插件目录复制到工程下的 `plugins/official/<folder>`（默认复制）。
- 子插件会递归检查其各自 manifest 的 `copy_on_project_save` 字段。

示例 manifest.lua
return {
id = "com.mydaw.gainfader",
name = "Gain Fader",
version = "0.1.0",
copy_on_project_save = true,
backend = {
type = "local",
path = "backend/target/release/libgain_fader_plugin.so",
},
frontend = {
path = "frontend/dist"
},
children = {
{
id = "com.mydaw.somechild",
backend = { type = "local", path = "../child_plugin/backend/target/release/libchild.so" }
}
}
}

注记

- 动态库后缀跨平台不同（.so/.dylib/.dll），建议 `path` 指向构建产物且 manifest 明确指明目标平台，或由宿主在加载时尝试常见后缀。
- 对于私有/闭源插件，作者可以将 `copy_on_project_save = false` 以避免将二进制包含到工程中（工程可在加载时回退到系统安装的插件）。
- 宿主也可以支持 `assets` 字段列出应该复制的额外文件。
