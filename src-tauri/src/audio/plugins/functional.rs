use crate::audio::core::plugin::{
    AudioBuffer, IOConfig, Plugin, PluginEvent, PluginInfo, PluginParameter, PluginType,
};
use uuid::Uuid;

#[allow(dead_code)]
pub struct FunctionalPlugin {
    id: Uuid,
    info: PluginInfo,
    io_config: IOConfig,
    params: Vec<PluginParameter>,
    process_fn:
        Box<dyn FnMut(&mut AudioBuffer, &[PluginEvent], &mut Vec<PluginEvent>) + Send + Sync>,
}

#[allow(dead_code)]
impl FunctionalPlugin {
    pub fn new(
        name: &str,
        unique_id: &str,
        process_fn: impl FnMut(&mut AudioBuffer, &[PluginEvent], &mut Vec<PluginEvent>)
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            info: PluginInfo {
                name: name.to_string(),
                vendor: "My DAW".to_string(),
                url: "".to_string(),
                plugin_type: PluginType::Native,
                unique_id: unique_id.to_string(),
            },
            io_config: IOConfig::default(),
            params: Vec::new(),
            process_fn: Box::new(process_fn),
        }
    }

    pub fn with_io(mut self, inputs: usize, outputs: usize) -> Self {
        self.io_config = IOConfig { inputs, outputs };
        self
    }

    pub fn with_param(mut self, param: PluginParameter) -> Self {
        self.params.push(param);
        self
    }
}

impl Plugin for FunctionalPlugin {
    fn info(&self) -> PluginInfo {
        self.info.clone()
    }

    fn get_io_config(&self) -> IOConfig {
        self.io_config.clone()
    }

    fn get_parameters(&self) -> Vec<PluginParameter> {
        self.params.clone()
    }

    fn process(
        &mut self,
        buffer: &mut AudioBuffer,
        events: &[PluginEvent],
        output_events: &mut Vec<PluginEvent>,
    ) {
        (self.process_fn)(buffer, events, output_events);
    }

    fn get_param(&self, _id: u32) -> f32 {
        0.0 // TODO: 存储参数状态
    }

    fn set_param(&mut self, _id: u32, _value: f32) {
        // TODO: 存储参数状态
    }
}
