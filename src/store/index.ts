import { createStore, produce } from "solid-js/store";
import { invoke } from "@tauri-apps/api/core";
import { ProjectStore, ClipContent, ClipInstance, Track } from "./types";

const DEFAULT_PROJECT: ProjectStore = {
    info: {
        name: "New Project",
        artist: "User",
        bpm: 120,
        timeSignature: [4, 4],
    },
    playback: {
        isPlaying: false,
        currentBar: 1,
        startTime: null,
    },
    tracks: [
        { id: "t1", name: "Grand Piano", color: "#aec6ff", muted: false, soloed: false },
        { id: "t2", name: "Drums", color: "#ffb4ab", muted: false, soloed: false },
        { id: "t3", name: "Bass", color: "#bfc6dc", muted: false, soloed: false },
    ],
    clips: [],
    clipLibrary: {},
    selectedTrackId: null,
};

export const [store, setStore] = createStore<ProjectStore>(DEFAULT_PROJECT);

// --- Actions ---

export const selectTrack = (id: string) => {
    setStore("selectedTrackId", id);
};

export const togglePlayback = async () => {
    try {
        const isPlaying = await invoke<boolean>("toggle_audio");
        setStore("playback", "isPlaying", isPlaying);
    } catch (e) {
        console.error("Failed to toggle audio:", e);
    }
};

export const setBpm = (bpm: number) => {
    setStore("info", "bpm", bpm);
};

export const addTrack = () => {
    const id = `t${Date.now()}`;
    const newTrack: Track = {
        id,
        name: `Track ${store.tracks.length + 1}`,
        color: "#e3e2e6",
        muted: false,
        soloed: false,
    };
    setStore("tracks", [...store.tracks, newTrack]);
};

// Helper to find or create content by name
const getOrCreateContentId = (name: string, color: string): string => {
    // Check if content with this name already exists
    const existingId = Object.keys(store.clipLibrary).find(
        (key) => store.clipLibrary[key].name === name
    );

    if (existingId) {
        return existingId;
    }

    // Create new
    const newId = `c_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    const newContent: ClipContent = {
        id: newId,
        name,
        color,
        notes: [],
    };

    setStore("clipLibrary", newId, newContent);
    return newId;
};

export const addClip = (trackId: string, startBar: number, name: string = "Clip", color: string = "#aec6ff") => {
    const contentId = getOrCreateContentId(name, color);

    const newClip: ClipInstance = {
        id: `ci_${Date.now()}_${Math.random()}`,
        trackId,
        clipContentId: contentId,
        startBar,
        lengthBars: 4, // Default 4 bars
    };

    setStore("clips", [...store.clips, newClip]);
};

export const renameClipContent = (contentId: string, newName: string) => {
    // Check if another content already has this name?
    // For now, just rename the content. All instances will reflect the new name.
    // If we wanted "copy on write" behavior when renaming to a new unique name, we'd do that here.
    // But user requirement: "modifying one modifies all", so renaming the content is correct.

    // However, if the user renames it to a name that ALREADY exists, maybe they want to link to THAT content?
    // That's a complex UX decision. For now, let's just rename the current content object.
    setStore("clipLibrary", contentId, "name", newName);
};

export const updateClipPosition = (instanceId: string, newStartBar: number) => {
    setStore("clips", (c) => c.id === instanceId, "startBar", newStartBar);
};
