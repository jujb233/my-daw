use crate::audio::core::plugin::Plugin;
use crate::audio::plugins::container::PluginContainer;
use crate::audio::plugins::gain_fader::GainFader;
use crate::audio::plugins::wave_generator::WaveGenerator;

pub fn create_simple_synth() -> Box<dyn Plugin> {
    let mut container = PluginContainer::new("Simple Synth", "com.mydaw.simplesynth");

    let wave = WaveGenerator::new();
    let gain = GainFader::new();

    let wave_idx = container.add_plugin(Box::new(wave));
    let gain_idx = container.add_plugin(Box::new(gain));

    // 将容器参数 0 映射到 GainFader 参数 0（增益）
    container.map_param(0, gain_idx, 0);

    // 将容器参数 1 映射到 WaveGenerator 参数 1（波形）
    container.map_param(1, wave_idx, 1);

    container.set_io_config(0, 2); // 0 个输入，2 个输出

    Box::new(container)
}
