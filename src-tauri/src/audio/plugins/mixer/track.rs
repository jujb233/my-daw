use crate::audio::core::plugin::{
        AudioBuffer, IOConfig, ParameterType, Plugin, PluginEvent, PluginInfo, PluginParameter, PluginType,
};
use crate::audio::plugins::mixer::level_meter::LevelMeter;
use std::collections::HashMap;
use uuid::Uuid;

pub struct MixerTrack {
        #[allow(dead_code)]
        pub id: Uuid,
        pub container: Box<dyn Plugin>,
        pub meter_id: Uuid,
        #[allow(dead_code)]
        pub fader_id: Uuid,
}

// Minimal in-file container replacement to avoid depending on removed host builtin module.
struct LocalContainer {
        plugins: Vec<Box<dyn Plugin>>,
        param_map: HashMap<u32, (usize, u32)>,
        info: PluginInfo,
}

impl LocalContainer {
        pub fn new(name: &str, unique_id: &str) -> Self {
                Self {
                        plugins: Vec::new(),
                        param_map: HashMap::new(),
                        info: PluginInfo {
                                name: name.to_string(),
                                vendor: "My DAW".to_string(),
                                url: "".to_string(),
                                plugin_type: PluginType::Native,
                                unique_id: unique_id.to_string(),
                        },
                }
        }

        pub fn add_plugin(&mut self, p: Box<dyn Plugin>) -> usize {
                self.plugins.push(p);
                self.plugins.len() - 1
        }

        pub fn map_param(&mut self, external_id: u32, plugin_index: usize, internal_id: u32) {
                self.param_map.insert(external_id, (plugin_index, internal_id));
        }
}

impl Plugin for LocalContainer {
        fn info(&self) -> PluginInfo {
                self.info.clone()
        }

        fn get_parameters(&self) -> Vec<PluginParameter> {
                let mut params = Vec::new();
                for (external_id, (plugin_idx, internal_id)) in &self.param_map {
                        if let Some(plugin) = self.plugins.get(*plugin_idx) {
                                let child_params = plugin.get_parameters();
                                if let Some(child_param) = child_params.iter().find(|p| p.id == *internal_id) {
                                        let mut p = child_param.clone();
                                        p.id = *external_id;
                                        params.push(p);
                                }
                        }
                }
                params.sort_by_key(|p| p.id);
                params
        }

        fn get_state(&self) -> Vec<u8> {
                Vec::new()
        }

        fn set_state(&mut self, _state: &[u8]) {}

        fn get_io_config(&self) -> IOConfig {
                IOConfig::default()
        }

        fn process(&mut self, buffer: &mut AudioBuffer, events: &[PluginEvent], output_events: &mut Vec<PluginEvent>) {
                // Apply parameter events directly
                for event in events {
                        if let PluginEvent::Parameter { id, value } = event {
                                if let Some((plugin_idx, internal_id)) = self.param_map.get(id) {
                                        if let Some(plugin) = self.plugins.get_mut(*plugin_idx) {
                                                plugin.set_param(*internal_id, *value);
                                        }
                                }
                        }
                }

                // Pass MIDI-only events to child plugins
                let midi_events: Vec<PluginEvent> = events
                        .iter()
                        .filter(|e| matches!(e, PluginEvent::Midi(_)))
                        .cloned()
                        .collect();

                for plugin in self.plugins.iter_mut() {
                        plugin.process(buffer, &midi_events, output_events);
                }
        }

        fn get_param(&self, id: u32) -> f32 {
                if let Some((plugin_idx, internal_id)) = self.param_map.get(&id) {
                        if let Some(plugin) = self.plugins.get(*plugin_idx) {
                                return plugin.get_param(*internal_id);
                        }
                }
                0.0
        }

        fn set_param(&mut self, id: u32, value: f32) {
                if let Some((plugin_idx, internal_id)) = self.param_map.get(&id) {
                        if let Some(plugin) = self.plugins.get_mut(*plugin_idx) {
                                plugin.set_param(*internal_id, value);
                        }
                }
        }
}

// Minimal gain plugin used by MixerTrack as a local replacement.
struct NoopGain {
        gain: f32,
}

impl NoopGain {
        pub fn new() -> Self {
                Self { gain: 0.5 }
        }
}

impl Plugin for NoopGain {
        fn info(&self) -> PluginInfo {
                PluginInfo {
                        name: "Gain Fader".to_string(),
                        vendor: "My DAW".to_string(),
                        url: "".to_string(),
                        plugin_type: PluginType::Native,
                        unique_id: "com.mydaw.gainfader".to_string(),
                }
        }

        fn get_parameters(&self) -> Vec<PluginParameter> {
                vec![PluginParameter {
                        id: 0,
                        name: "Gain".to_string(),
                        min_value: 0.0,
                        max_value: 1.0,
                        default_value: 0.5,
                        value_type: ParameterType::Float,
                }]
        }

        fn get_state(&self) -> Vec<u8> {
                self.gain.to_le_bytes().to_vec()
        }

        fn set_state(&mut self, state: &[u8]) {
                if state.len() >= 4 {
                        let bytes: [u8; 4] = state[0..4].try_into().unwrap();
                        self.gain = f32::from_le_bytes(bytes);
                }
        }

        fn get_io_config(&self) -> IOConfig {
                IOConfig::default()
        }

        fn process(
                &mut self,
                buffer: &mut AudioBuffer,
                _events: &[PluginEvent],
                _output_events: &mut Vec<PluginEvent>,
        ) {
                for sample in buffer.samples.iter_mut() {
                        *sample *= self.gain;
                }
        }

        fn get_param(&self, id: u32) -> f32 {
                if id == 0 { self.gain } else { 0.0 }
        }

        fn set_param(&mut self, id: u32, value: f32) {
                if id == 0 {
                        self.gain = value;
                }
        }
}

impl MixerTrack {
        pub fn new(meter_id: Option<Uuid>) -> Self {
                let mut container = LocalContainer::new("Mixer Track", "com.mydaw.mixertrack");

                // 添加电平计（Meter）的位置：前推子（Pre-fader）或后推子（Post-fader）？通常通道条是 Post-fader，但我们在此用于输入监控使用 Pre-fader。
                // 用户要求：左边为电平计，右边为推子。通常电平计显示信号电平。
                // 我们将电平计放在 Fader 之后以显示轨道的输出电平。

                // 实际标准通常是：
                // 插入效果 -> 推子 -> 电平计（Post-Fader）
                // 或者
                // 插入效果 -> 电平计（Pre-Fader） -> 推子

                // 我们选择：推子 -> 电平计。

                let fader = NoopGain::new();
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
                        container: Box::new(container),
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
