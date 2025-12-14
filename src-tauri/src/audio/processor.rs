#[allow(dead_code)]
/// 处理上下文（当前仅包含采样率）
pub struct ProcessContext {
        pub sample_rate: f32,
}

/// 音频处理节点接口：将处理结果写入 `output`（交错格式），并可使用 `context`。
#[allow(dead_code)]
pub trait AudioProcessor: Send + Sync {
        fn process(&mut self, output: &mut [f32], channels: usize, context: &ProcessContext);
}
