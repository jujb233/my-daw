use crate::audio::core::plugin::{
    AudioBuffer, IOConfig, Plugin, PluginEvent, PluginInfo, PluginParameter, PluginType,
};
use std::collections::HashMap;
use uuid::Uuid;

pub struct PluginContainer {
    #[allow(dead_code)]
    id: Uuid,
    plugins: Vec<Box<dyn Plugin>>,
    // 映射 external_param_id -> (plugin_index, internal_param_id)
    param_map: HashMap<u32, (usize, u32)>,
    info: PluginInfo,
    io_config: IOConfig,
}

impl PluginContainer {
    pub fn new(name: &str, unique_id: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            plugins: Vec::new(),
            param_map: HashMap::new(),
            info: PluginInfo {
                name: name.to_string(),
                vendor: "My DAW".to_string(),
                url: "".to_string(),
                plugin_type: PluginType::Native,
                unique_id: unique_id.to_string(),
            },
            io_config: IOConfig::default(),
        }
    }

    pub fn set_io_config(&mut self, inputs: usize, outputs: usize) {
        self.io_config = IOConfig { inputs, outputs };
    }

    pub fn add_plugin(&mut self, plugin: Box<dyn Plugin>) -> usize {
        self.plugins.push(plugin);
        self.plugins.len() - 1
    }

    #[allow(dead_code)]
    pub fn insert_plugin(&mut self, index: usize, plugin: Box<dyn Plugin>) {
        if index <= self.plugins.len() {
            self.plugins.insert(index, plugin);
            // 注意：这会使 param_map 中现有的索引失效！
            // 对于此原型，我们假设一次性构建链条或进行重映射。
            // 由于我们在更改时会重建整个引擎，因此这没问题。
        }
    }

    pub fn map_param(&mut self, external_id: u32, plugin_index: usize, internal_id: u32) {
        self.param_map
            .insert(external_id, (plugin_index, internal_id));
    }
}

impl Plugin for PluginContainer {
    fn info(&self) -> PluginInfo {
        self.info.clone()
    }

    fn get_io_config(&self) -> IOConfig {
        self.io_config.clone()
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

    fn process(
        &mut self,
        buffer: &mut AudioBuffer,
        events: &[PluginEvent],
        output_events: &mut Vec<PluginEvent>,
    ) {
        // 切分事件：全局事件（MIDI）是否广播给所有插件或特定插件？
        // 参数事件需要基于 param_map 进行路由。

        // 目前，先把 MIDI 广播给所有插件，并路由参数事件。
        // 但我们无法在不分配内存的情况下轻易拆分事件切片。
        // 因此我们会遍历插件，并为每个插件筛选相关事件。
        // 这对大量事件来说效率较低，但暂时可接受。

        // 优化：将事件预处理成每个插件的列表？
        // 或者将所有事件传递给所有插件，但插件只对它们识别的事件做出反应？
        // 但插件并不知道容器的映射信息。
        // 所以容器必须翻译这些事件。

        // 如果我们要翻译参数，则需要为每个插件构建新的事件列表。
        // 由于在音频线程中不宜分配内存，这很棘手。
        // 理想情况下，我们应该有一个预先分配的事件缓冲区。

        // 对于这个原型，我们采用简化的方法：
        // 1. 立即将参数更改应用到插件（直接修改状态）。
        // 2. 仅将 MIDI 事件传给 process()。

        // 提取参数更改
        for event in events {
            if let PluginEvent::Parameter { id, value } = event {
                if let Some((plugin_idx, internal_id)) = self.param_map.get(id) {
                    if let Some(plugin) = self.plugins.get_mut(*plugin_idx) {
                        plugin.set_param(*internal_id, *value);
                    }
                }
            }
        }

        // 通过插件链传递音频
        for plugin in &mut self.plugins {
            // 我们应该在此处过滤事件，仅包含 MIDI 或已翻译的参数事件。
            // 但既然我们已通过 set_param 应用了参数，也许仅传递 MIDI 即可？
            // 有些插件可能期望通过事件获得采样精度的参数自动化。
            // 目前，就传递原始事件（可能包含子插件无法识别的 ID）
            // 但应过滤掉参数以避免混淆？
            // 或者更好地做法：子插件不知道容器的 ID 列表。
            // 所以我们可能只应传递 MIDI 事件。

            let midi_events: Vec<PluginEvent> = events
                .iter()
                .filter(|e| matches!(e, PluginEvent::Midi(_)))
                .cloned()
                .collect(); // 在音频线程上分配内存！不理想，但用于原型可接受。

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
