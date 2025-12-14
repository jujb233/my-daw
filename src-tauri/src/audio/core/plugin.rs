use serde::{Deserialize, Serialize};

/// 简单的 MIDI 事件：按下 / 释放
#[derive(Debug, Clone, Copy)]
pub enum NoteEvent {
        NoteOn {
                #[allow(dead_code)]
                note: u8,
                #[allow(dead_code)]
                velocity: f32,
        },
        NoteOff {
                #[allow(dead_code)]
                note: u8,
        },
}

#[derive(Debug, Clone)]
/// 插件运行时事件：包含 MIDI、参数变化、传输状态等
pub enum PluginEvent {
        Midi(NoteEvent),
        Parameter {
                id: u32,
                value: f32,
        },
        Transport {
                // 是否正在播放
                playing: bool,
                // 可选播放位置（秒）
                position: Option<f64>,
                // 可选节拍（BPM）
                tempo: Option<f64>,
        },
        #[allow(dead_code)]
        Custom(String),
}

/// 音频缓冲区借用：交错样本数组、通道数与采样率
pub struct AudioBuffer<'a> {
        pub samples: &'a mut [f32],
        pub channels: usize,
        pub sample_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 插件 I/O 通道配置
pub struct IOConfig {
        pub inputs: usize,
        pub outputs: usize,
}

impl Default for IOConfig {
        fn default() -> Self {
                Self { inputs: 2, outputs: 2 }
        }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 插件类型（Native / Clap / VST 等）
pub enum PluginType {
        Native,
        Clap,
        Vst,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 参数类型定义
pub enum ParameterType {
        Float,
        Int,
        Bool,
        Enum(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 单参数描述：ID、名称、范围与类型
pub struct PluginParameter {
        pub id: u32,
        pub name: String,
        pub min_value: f32,
        pub max_value: f32,
        pub default_value: f32,
        pub value_type: ParameterType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
/// 插件静态元信息，用于 UI 显示与识别
pub struct PluginInfo {
        pub name: String,
        pub vendor: String,
        pub url: String,
        pub plugin_type: PluginType,
        pub unique_id: String,
}

/// 插件运行时接口：处理音频块、管理参数与（可选）序列化状态
pub trait Plugin: Send + Sync {
        /// 静态元信息
        fn info(&self) -> PluginInfo;

        /// 返回参数列表
        fn get_parameters(&self) -> Vec<PluginParameter>;

        /// 可选：序列化运行时状态
        fn get_state(&self) -> Vec<u8> {
                Vec::new()
        }

        /// 可选：反序列化运行时状态
        fn set_state(&mut self, _state: &[u8]) {}

        /// 可选：I/O 通道配置（默认立体声）
        fn get_io_config(&self) -> IOConfig {
                IOConfig::default()
        }

        /// 核心处理：就地修改 `buffer.samples`，并可读取 `events`、产生 `output_events`
        fn process(&mut self, buffer: &mut AudioBuffer, events: &[PluginEvent], output_events: &mut Vec<PluginEvent>);

        /// 参数访问接口
        fn get_param(&self, id: u32) -> f32;
        fn set_param(&mut self, id: u32, value: f32);
}
