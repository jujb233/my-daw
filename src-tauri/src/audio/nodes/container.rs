use uuid::Uuid;
use crate::audio::plugin::{Plugin, AudioBuffer, PluginEvent};

pub struct PluginContainer {
    id: Uuid,
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginContainer {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            plugins: Vec::new(),
        }
    }

    pub fn add_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }
}

impl Plugin for PluginContainer {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        "Plugin Container"
    }

    fn process(&mut self, buffer: &mut AudioBuffer, events: &[PluginEvent]) {
        // Pass audio through the chain
        // Note: In a real DAW, we might want to split events per plugin, 
        // but for now we broadcast events to all plugins in the chain.
        for plugin in &mut self.plugins {
            plugin.process(buffer, events);
        }
    }

    fn get_param(&self, _id: u32) -> f32 {
        0.0
    }

    fn set_param(&mut self, _id: u32, _value: f32) {
        // Simple routing: if we had a way to address specific plugins, we would.
        // For now, maybe broadcast? Or just ignore.
    }
}
