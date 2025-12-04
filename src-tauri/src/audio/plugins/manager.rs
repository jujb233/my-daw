use crate::audio::core::plugin::{Plugin, PluginInfo};
use crate::audio::plugins::clap::plugin::ClapPlugin;
use crate::audio::plugins::gain_fader::GainFader;
use crate::audio::plugins::mixer::level_meter::LevelMeter;
use crate::audio::plugins::simple_synth::create_simple_synth;
use crate::audio::plugins::wave_generator::WaveGenerator;
use std::collections::HashMap;

pub struct PluginManager {
    known_plugins: HashMap<String, PluginInfo>,
    clap_paths: HashMap<String, String>,
}

impl PluginManager {
    pub fn new() -> Self {
        let mut manager = Self {
            known_plugins: HashMap::new(),
            clap_paths: HashMap::new(),
        };
        manager.scan_native_plugins();
        manager
    }

    fn scan_native_plugins(&mut self) {
        // 简易合成器 (Simple Synth)
        let synth = create_simple_synth();
        let info = synth.info();
        self.known_plugins.insert(info.unique_id.clone(), info);

        // 波形生成器 (Wave Generator)
        let wave = WaveGenerator::new();
        let info = wave.info();
        self.known_plugins.insert(info.unique_id.clone(), info);

        // 增益推子 (Gain Fader)
        let gain = GainFader::new();
        let info = gain.info();
        self.known_plugins.insert(info.unique_id.clone(), info);

        // 电平计 (Level Meter)
        let meter = LevelMeter::new();
        let info = meter.info();
        self.known_plugins.insert(info.unique_id.clone(), info);
    }

    pub fn scan_clap_plugin(&mut self, path: &str) -> Result<PluginInfo, String> {
        unsafe {
            let plugin = ClapPlugin::new(path)?;
            let info = plugin.info();
            self.known_plugins
                .insert(info.unique_id.clone(), info.clone());
            self.clap_paths
                .insert(info.unique_id.clone(), path.to_string());
            Ok(info)
        }
    }

    pub fn get_available_plugins(&self) -> Vec<PluginInfo> {
        self.known_plugins.values().cloned().collect()
    }

    pub fn create_plugin(&self, unique_id: &str) -> Option<Box<dyn Plugin>> {
        if unique_id == "com.mydaw.simplesynth" {
            return Some(create_simple_synth());
        }
        if unique_id == "com.mydaw.wavegenerator" {
            return Some(Box::new(WaveGenerator::new()));
        }
        if unique_id == "com.mydaw.gainfader" {
            return Some(Box::new(GainFader::new()));
        }
        if unique_id == "com.mydaw.levelmeter" {
            return Some(Box::new(LevelMeter::new()));
        }

        if let Some(path) = self.clap_paths.get(unique_id) {
            unsafe {
                if let Ok(plugin) = ClapPlugin::new(path) {
                    return Some(Box::new(plugin));
                }
            }
        }

        None
    }

    pub fn get_plugin_parameters(
        &self,
        unique_id: &str,
    ) -> Option<Vec<crate::audio::core::plugin::PluginParameter>> {
        if let Some(plugin) = self.create_plugin(unique_id) {
            return Some(plugin.get_parameters());
        }
        None
    }
}
