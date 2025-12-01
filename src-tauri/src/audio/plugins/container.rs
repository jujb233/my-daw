use uuid::Uuid;
use crate::audio::core::plugin::{Plugin, AudioBuffer, PluginEvent, PluginInfo, PluginType};
use std::collections::HashMap;

pub struct PluginContainer {
    id: Uuid,
    plugins: Vec<Box<dyn Plugin>>,
    // Map external_param_id -> (plugin_index, internal_param_id)
    param_map: HashMap<u32, (usize, u32)>,
}

impl PluginContainer {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            plugins: Vec::new(),
            param_map: HashMap::new(),
        }
    }

    pub fn add_plugin(&mut self, plugin: Box<dyn Plugin>) -> usize {
        self.plugins.push(plugin);
        self.plugins.len() - 1
    }

    pub fn map_param(&mut self, external_id: u32, plugin_index: usize, internal_id: u32) {
        self.param_map.insert(external_id, (plugin_index, internal_id));
    }
}

impl Plugin for PluginContainer {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Plugin Container".to_string(),
            vendor: "My DAW".to_string(),
            url: "".to_string(),
            plugin_type: PluginType::Native,
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer, events: &[PluginEvent]) {
        // Split events: Global events (Midi) go to everyone (or specific ones?)
        // Parameter events need to be routed based on param_map.
        
        // For now, let's just broadcast MIDI to all, and route Parameters.
        // But wait, we can't easily split the slice of events without allocation.
        // So we'll iterate plugins and for each plugin, filter relevant events.
        // This is inefficient for many events, but fine for now.
        
        // Optimization: Pre-process events into a list for each plugin?
        // Or just pass all events to all plugins, but plugins only react to what they know?
        // But plugins don't know about the Container's mapping.
        // So the Container MUST translate the event.
        
        // We need to construct a new event list for each plugin if we want to translate parameters.
        // Since we can't easily allocate on the audio thread, this is tricky.
        // Ideally, we'd have a pre-allocated event buffer.
        
        // For this prototype, we'll do a simplified approach:
        // 1. Apply parameter changes immediately to the plugins (mutate state).
        // 2. Pass only MIDI events to process().
        
        // Extract parameter changes
        for event in events {
            if let PluginEvent::Parameter { id, value } = event {
                if let Some((plugin_idx, internal_id)) = self.param_map.get(id) {
                    if let Some(plugin) = self.plugins.get_mut(*plugin_idx) {
                        plugin.set_param(*internal_id, *value);
                    }
                }
            }
        }

        // Pass audio through the chain
        for plugin in &mut self.plugins {
            // We should filter events to only include MIDI here, or translated params.
            // But since we already applied params via set_param, maybe we just pass MIDI?
            // Some plugins might expect sample-accurate parameter automation via events.
            // For now, let's just pass the original events (which might have wrong IDs for children)
            // BUT filter out parameters to avoid confusion?
            // Or better: The children don't know the container's IDs.
            // So we should probably pass only MIDI events.
            
            let midi_events: Vec<PluginEvent> = events.iter()
                .filter(|e| matches!(e, PluginEvent::Midi(_)))
                .cloned()
                .collect(); // Allocation on audio thread! Bad! But acceptable for prototype.
            
            plugin.process(buffer, &midi_events);
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
