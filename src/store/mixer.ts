import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

export interface MixerTrackData {
    id: number;
    label: string;
    volume: number;
    pan: number;
    mute: boolean;
    solo: boolean;
    meter_id?: string;
}

export const [mixerTracks, setMixerTracks] = createSignal<MixerTrackData[]>([]);
export const [meterLevels, setMeterLevels] = createSignal<{ [key: string]: number }>({});

export const fetchMixerTracks = async () => {
    try {
        const tracks = await invoke<MixerTrackData[]>("get_mixer_tracks");
        setMixerTracks(tracks);
    } catch (e) {
        console.error("Failed to fetch mixer tracks:", e);
    }
};

export const addMixerTrack = async () => {
    try {
        await invoke("add_mixer_track");
        await fetchMixerTracks();
    } catch (e) {
        console.error("Failed to add mixer track:", e);
    }
};

export const removeMixerTrack = async (index: number) => {
    try {
        await invoke("remove_mixer_track", { index });
        await fetchMixerTracks();
    } catch (e) {
        console.error("Failed to remove mixer track:", e);
    }
};

export const setTrackVolume = async (trackId: number, volume: number) => {
    // Update local state immediately for responsiveness
    setMixerTracks(prev => prev.map(t => t.id === trackId ? { ...t, volume } : t));

    // Send to backend
    // Param ID scheme: TrackID * 100 + 0 (Fader Gain is param 0)
    const paramId = trackId * 100 + 0;
    try {
        await invoke("update_parameter", { paramId, value: volume });
    } catch (e) {
        console.error("Failed to update track volume:", e);
    }
};

export const toggleMute = async (trackId: number) => {
    setMixerTracks(prev => prev.map(t => t.id === trackId ? { ...t, mute: !t.mute } : t));
    // TODO: Send to backend
};

export const toggleSolo = async (trackId: number) => {
    setMixerTracks(prev => prev.map(t => t.id === trackId ? { ...t, solo: !t.solo } : t));
    // TODO: Send to backend
};

// Polling for meter levels
let pollInterval: number | undefined;

export const startMetering = () => {
    if (pollInterval) return;
    pollInterval = window.setInterval(async () => {
        try {
            const levels = await invoke<{ [key: string]: number }>("get_meter_levels_cmd");
            setMeterLevels(levels);
        } catch (e) {
            console.error("Failed to fetch meter levels:", e);
        }
    }, 50); // 20fps
};

export const stopMetering = () => {
    if (pollInterval) {
        clearInterval(pollInterval);
        pollInterval = undefined;
    }
};
