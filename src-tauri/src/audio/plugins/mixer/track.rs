use crate::audio::core::plugin::{AudioBuffer, Plugin, PluginEvent};
use crate::audio::plugins::container::PluginContainer;
use crate::audio::plugins::gain_fader::GainFader;
use crate::audio::plugins::mixer::level_meter::LevelMeter;
use uuid::Uuid;

pub struct MixerTrack {
    #[allow(dead_code)]
    pub id: Uuid,
    pub container: PluginContainer,
    pub meter_id: Uuid,
    #[allow(dead_code)]
    pub fader_id: Uuid,
}

impl MixerTrack {
    pub fn new(meter_id: Option<Uuid>) -> Self {
        let mut container = PluginContainer::new("Mixer Track", "com.mydaw.mixertrack");

        // 添加电平计（Meter）的位置：前推子（Pre-fader）或后推子（Post-fader）？通常通道条是 Post-fader，但我们在此用于输入监控使用 Pre-fader。
        // 用户要求：左边为电平计，右边为推子。通常电平计显示信号电平。
        // 我们将电平计放在 Fader 之后以显示轨道的输出电平。

        // 实际标准通常是：
        // 插入效果 -> 推子 -> 电平计（Post-Fader）
        // 或者
        // 插入效果 -> 电平计（Pre-Fader） -> 推子

        // 我们选择：推子 -> 电平计。

        let fader = GainFader::new();
        // 我们需要获取 ID 来映射参数，但 GainFader 在 new() 中不直接暴露 ID。
        // 直接添加即可。
        let fader_idx = container.add_plugin(Box::new(fader));

        let meter = if let Some(id) = meter_id {
            LevelMeter::with_id(id)
        } else {
            LevelMeter::new()
        };
        let meter_id = meter.get_id();
        let _meter_idx = container.add_plugin(Box::new(meter));

        // 将推子增益（参数 0）映射到轨道参数 0
        container.map_param(0, fader_idx, 0);

        Self {
            id: Uuid::new_v4(),
            container,
            meter_id,
            fader_id: Uuid::nil(), // 我们没有容易获取的 fader ID，但我们已将其映射到参数 0
        }
    }

    #[allow(dead_code)]
    pub fn process(
        &mut self,
        buffer: &mut AudioBuffer,
        events: &[PluginEvent],
        output_events: &mut Vec<PluginEvent>,
    ) {
        self.container.process(buffer, events, output_events);
    }
}
