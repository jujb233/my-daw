Plugin manifest (manifest.lua) — 规范

目的

- 描述原生插件的元信息、后端与可选前端路径、复制策略和子插件结构，便于宿主自动发现、加载与在工程保存时复现。

```markdown
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

参数声明（可选，但强烈建议）

为了使宿主能直接解析并在内建 UI 或插件自带 UI 中使用参数，插件作者应在 `manifest.lua` 中声明 `parameters` 字段。
`parameters` 应为一个数组，数组元素为包含参数元信息的 table。每个参数对象推荐字段：

- `id` (number) — 在插件内部唯一标识参数的整数 ID；若省略，宿主可用数组索引作为回退 ID。
- `name` (string) — 在 UI 中显示的参数名。
- `min` (number) — 参数最小值。
- `max` (number) — 参数最大值。
- `default` (number) — 参数默认值。
- `type` or `value_type` (string or array) — 参数类型：`"Float"`, `"Int"`, `"Bool"`，或直接用字符串数组表示枚举项（例如 `{ "Low", "Mid", "High" }`）。

宿主也会尝试从插件后端导出的 `plugin_info_json` 解析参数，但将 manifest 中的 `parameters` 作为首选或补充信息可以提高兼容性并简化工具链。

示例：在 manifest 中声明参数

return {
id = "com.mydaw.gainfader",
name = "Gain Fader",
version = "0.1.0",
copy_on_project_save = true,
backend = {
type = "local",
path = "backend/target/release/libgain_fader_plugin.so",
},
frontend = { path = "frontend/dist" },
parameters = {
{ id = 0, name = "Gain (dB)", min = -60.0, max = 12.0, default = 0.0, type = "Float" },
{ id = 1, name = "Bypass", min = 0, max = 1, default = 0, type = "Bool" },
{ id = 2, name = "Mode", min = 0, max = 2, default = 0, type = { "Low", "Mid", "High" } },
},
}

注：

- 如果插件既在 manifest 中声明了 `parameters`，又通过 `plugin_info_json` 返回参数，宿主可合并或以 manifest 为准；建议保持两者同步。
- 参数 ID 在插件不同版本间应尽量保持稳定，便于工程保存/恢复时正确映射。
```
