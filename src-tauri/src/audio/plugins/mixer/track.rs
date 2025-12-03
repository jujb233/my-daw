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

        // Add Meter (Pre-fader or Post-fader? Usually Post-fader for channel strip, but let's do Pre-fader for input monitoring?
        // User said: Left is Meter, Right is Fader. Usually Meter shows the signal level.
        // Let's put Meter AFTER Fader so it shows the output level of the track.

        // Actually, standard is:
        // Insert Effects -> Fader -> Meter (Post-Fader)
        // OR
        // Insert Effects -> Meter (Pre-Fader) -> Fader

        // Let's do: Fader -> Meter.

        let fader = GainFader::new();
        // We need to get the ID to map params, but GainFader doesn't expose ID easily in new().
        // We'll just add it.
        let fader_idx = container.add_plugin(Box::new(fader));

        let meter = if let Some(id) = meter_id {
            LevelMeter::with_id(id)
        } else {
            LevelMeter::new()
        };
        let meter_id = meter.get_id();
        let _meter_idx = container.add_plugin(Box::new(meter));

        // Map Fader Gain (Param 0) to Track Param 0
        container.map_param(0, fader_idx, 0);

        Self {
            id: Uuid::new_v4(),
            container,
            meter_id,
            fader_id: Uuid::nil(), // We don't have fader ID easily, but we mapped it to param 0
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
