use serde::{Deserialize, Serialize};

/// 简化的 MIDI 事件：按下与释放
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
/// 插件运行时事件（输入/输出、参数、传输状态等）
pub enum PluginEvent {
        Midi(NoteEvent),
        Parameter {
                id: u32,
                value: f32,
        },
        Transport {
                // 是否播放中
                playing: bool,
                // 可选播放位置（秒）
                position: Option<f64>,
                // 可选节拍（BPM）
                tempo: Option<f64>,
        },
        #[allow(dead_code)]
        Custom(String),
}

/// 音频缓冲区引用：对回调提供 samples 的可变借用
pub struct AudioBuffer<'a> {
        pub samples: &'a mut [f32],
        pub channels: usize,
        pub sample_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 插件的输入/输出通道配置
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
/// 插件类型标识（本地/Clap/VST 等）
pub enum PluginType {
        Native,
        Clap,
        Vst,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 参数的数据类型
pub enum ParameterType {
        Float,
        Int,
        Bool,
        Enum(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// 参数描述：ID、名称、范围与类型
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
/// 插件静态信息（用于 UI/识别）
pub struct PluginInfo {
        pub name: String,
        pub vendor: String,
        pub url: String,
        pub plugin_type: PluginType,
        pub unique_id: String,
}

/// 插件必须实现的运行时接口
pub trait Plugin: Send + Sync {
        #[allow(dead_code)]
        /// 静态信息（名称、厂商、类型等）
        fn info(&self) -> PluginInfo;

        /// 返回插件参数列表
        fn get_parameters(&self) -> Vec<PluginParameter>;

        /// 序列化运行时状态（可选）
        fn get_state(&self) -> Vec<u8> {
                Vec::new()
        }

        /// 反序列化运行时状态（可选）
        fn set_state(&mut self, _state: &[u8]) {}

        #[allow(dead_code)]
        /// 输入/输出通道配置，默认立体声 I/O
        fn get_io_config(&self) -> IOConfig {
                IOConfig::default()
        }

        /// 处理一个音频块：就地修改 `buffer.samples`，读取 `events`，并将任何输出事件追加到 `output_events`。
        fn process(&mut self, buffer: &mut AudioBuffer, events: &[PluginEvent], output_events: &mut Vec<PluginEvent>);

        // 参数访问
        #[allow(dead_code)]
        fn get_param(&self, id: u32) -> f32;
        fn set_param(&mut self, id: u32, value: f32);
}
