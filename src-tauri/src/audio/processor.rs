#[allow(dead_code)]
pub struct ProcessContext {
    pub sample_rate: f32,
}

/// 表示音频图中可生成或处理音频的节点的 trait。
#[allow(dead_code)]
pub trait AudioProcessor: Send + Sync {
    /// 处理音频并填充输出缓冲区。
    /// 当通道数 > 1 时，缓冲区为交错格式（interleaved）。
    /// 例如：立体声为 [L, R, L, R, ...]。
    fn process(&mut self, output: &mut [f32], channels: usize, context: &ProcessContext);
}
