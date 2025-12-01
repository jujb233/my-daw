use crate::audio::core::plugin::Plugin;
use crate::audio::plugins::container::PluginContainer;
use crate::audio::plugins::gain_fader::GainFader;
use crate::audio::plugins::wave_generator::WaveGenerator;

pub fn create_simple_synth() -> Box<dyn Plugin> {
    let mut container = PluginContainer::new();

    let wave = WaveGenerator::new();
    let gain = GainFader::new();

    let wave_idx = container.add_plugin(Box::new(wave));
    let gain_idx = container.add_plugin(Box::new(gain));

    // Map Container Param 0 -> GainFader Param 0 (Gain)
    container.map_param(0, gain_idx, 0);
    
    // Map Container Param 1 -> WaveGenerator Param 1 (Waveform)
    container.map_param(1, wave_idx, 1);

    Box::new(container)
}
