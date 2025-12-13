#[allow(dead_code)]
/// 处理上下文：包含运行时参数（当前仅采样率）。
pub struct ProcessContext {
        pub sample_rate: f32,
}

/// 音频处理节点接口。实现者应填充输出缓冲区并使用提供的上下文。
#[allow(dead_code)]
pub trait AudioProcessor: Send + Sync {
        /// 将处理结果写入 `output`（交错格式，当 `channels` > 1 时）。
        fn process(&mut self, output: &mut [f32], channels: usize, context: &ProcessContext);
}
