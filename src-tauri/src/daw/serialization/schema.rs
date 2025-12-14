use serde::{Deserialize, Serialize};

/// 项目序列化类型集合
///
/// 这些类型用于在磁盘（Lua 脚本 + SQLite 数据库）与内存中应用的模型之间进行映射。
/// - Lua 文件用于以可读、可编辑的形式导出项目结构（轨道/片段/插件等）
/// - SQLite 用于存储大量的 MIDI note 与二进制的插件状态（blob），以减小 Lua 导出产物的体积
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSchema {
        pub meta: ProjectMetadata,
        pub settings: ProjectSettings,
        pub tracks: Vec<TrackSchema>,
        pub mixer: MixerSchema,
        pub plugins: Vec<PluginSchema>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// 项目元信息（用户可读的元数据）
pub struct ProjectMetadata {
        /// 项目名称
        pub name: String,
        /// 作者
        pub author: String,
        /// 项目版本号（可选，便于脚本/导出/兼容性判断）
        pub version: String,
        /// 创建时间（Unix timestamp）
        pub created_at: u64,
        /// 更新时间（Unix timestamp）
        pub updated_at: u64,
        /// 项目描述/备注
        pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// 项目设置（节拍 / 采样率 / 拍号）
pub struct ProjectSettings {
        /// 当前 BPM（节拍/分钟）
        pub bpm: f64,
        /// 采样率（Hz）
        pub sample_rate: u32,
        /// 拍号（分子, 分母），例如 (4,4)
        pub time_signature: (u32, u32),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// 单个轨道的序列化表示（Arrangement Track）
pub struct TrackSchema {
        /// 轨道 id（在导出/导入中用于匹配轨道关系）
        pub id: usize,
        /// 轨道名称（用于 UI 显示）
        pub name: String,
        /// 轨道颜色（如 Hex 字符串）
        pub color: String,
        /// 轨道类型（Audio / Midi / Group / Return 等）
        pub track_type: TrackType,
        /// 该轨道包含的片段（clips 列表）
        pub clips: Vec<ClipSchema>,
        /// 此轨道输出目标的混音器 track id（用于路由）
        pub target_mixer_track_id: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// 轨道类型
pub enum TrackType {
        Audio,
        Midi,
        Group,
        Return,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// 片段（Clip）的序列化结构
///
/// 包含秒级与节拍级的时间信息，以便在重新载入时精准恢复 clip 的位置与长度。
pub struct ClipSchema {
        /// 片段 id（全局唯一字符串）
        pub id: String,
        /// 片段名称
        pub name: String,
        /// 片段颜色
        pub color: String,
        /// 片段起始位置（以秒为单位）
        pub start_position: f64,
        /// 起始小节/节拍/16 分音符/节拍 tick
        pub start_bar: u32,
        pub start_beat: u32,
        pub start_sixteenth: u32,
        pub start_tick: u32,
        /// 时长（秒）
        pub duration: f64,
        /// 时长（以 bars/beats/16 各层次计）
        pub duration_bars: u32,
        pub duration_beats: u32,
        pub duration_sixteenths: u32,
        pub duration_ticks: u32,
        /// 总 ticks（用于精确相对 timing）
        pub duration_total_ticks: u64,
        /// 音频片段偏移量（只对音频片段生效，单位秒）
        pub offset: f64,
        /// 内容类型：Midi 或 Audio（包含文件路径）
        pub content_type: ClipContentType,
        /// 音符数量（用于快速统计，note 本身按需存储在 `notes`）：
        /// - 当内容为 MIDI 时，`notes` 会包含对应的 NoteSchema。
        pub note_count: usize,
        pub notes: Vec<NoteSchema>,
        /// 该 clip 使用的 instrument id 列表
        pub instrument_ids: Vec<String>,
        /// instrument routing（map instrument_id -> target track id）
        pub instrument_routes: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// 片段的内容类型（MIDI 或 Audio 文件引用）
pub enum ClipContentType {
        /// MIDI 类型：clip 中包含 note 列表
        Midi,
        /// Audio 类型：引用外部音频文件，`file_path` 为绝对或相对路径
        Audio { file_path: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// MIDI note 的序列化结构（相对于 clip 起始点的时间）
pub struct NoteSchema {
        /// MIDI 音高（0-127）
        pub note: u8,
        /// 音符相对于 clip 起始点的起始时间（秒）
        pub start: f64,
        /// 音符持续时间（秒）
        pub duration: f64,
        /// 音量（0.0 - 1.0）或以 0-127 值归一化后的浮点值
        pub velocity: f32,
        /// MIDI 通道（0-15）
        pub channel: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// 混音器序列化结构：包含全部混音器轨道信息
pub struct MixerSchema {
        pub tracks: Vec<MixerTrackSchema>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// 单个混音轨道的序列化结构（volume/pan/mute/solo 等）
pub struct MixerTrackSchema {
        /// 混音轨道 id
        pub id: usize,
        /// 音量（-1.0 - 1.0 或 0.0 - 1.0，视实现而定）
        pub volume: f32,
        /// 声像（-1.0 左，1.0 右）
        pub pan: f32,
        /// 静音标志
        pub mute: bool,
        /// solo 标志
        pub solo: bool,
        /// 该混音轨道上插入的 plugin instance id 列表
        pub plugin_instances: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// 插件在项目中的元数据（用于生成 `project.lua` 并在 `data.db` 中查找对应状态 blob）
pub struct PluginSchema {
        /// 插件实例 ID（在项目中唯一标识）
        pub id: String,
        /// 插件显示名称
        pub name: String,
        /// 插件 label（UI/实例显示识别名）
        pub label: String,
        /// 插件的路由目标（所在的 track index）
        pub routing_track_index: usize,
        /// 插件格式标识（例如：CLAP/Local/Builtin 等）
        pub format: String,
        /// 在 SQLite `plugins` 表中对应的状态 blob 的 id（如有）
        pub state_blob_id: Option<i64>,
}
