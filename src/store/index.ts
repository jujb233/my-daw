import { createStore } from "solid-js/store";
import { DawService } from "../services/daw";
import { ProjectStore, ClipContent, ClipInstance, Track, Note } from "./types";

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
        { id: 0, name: "Grand Piano", color: "#aec6ff", muted: false, soloed: false },
        { id: 1, name: "Drums", color: "#ffb4ab", muted: false, soloed: false },
        { id: 2, name: "Bass", color: "#bfc6dc", muted: false, soloed: false },
    ],
    clips: [],
    clipLibrary: {},
    selectedTrackId: null,
    selectedClipId: null,
};

export const [store, setStore] = createStore<ProjectStore>(DEFAULT_PROJECT);

// --- Actions ---

export const selectClip = (id: number | null) => {
    setStore("selectedClipId", id);
};

export const selectTrack = (id: number) => {
    setStore("selectedTrackId", id);
};

let animationFrameId: number | null = null;

const updatePlaybackState = async () => {
    try {
        const [isPlaying, time] = await DawService.getPlaybackState();

        const bpm = store.info.bpm;
        const timeSigNum = store.info.timeSignature[0];

        const currentBeat = time * (bpm / 60);
        const currentBar = (currentBeat / timeSigNum) + 1;

        // Only update if we are actually playing or if we just stopped
        // If local state says playing, but backend says not yet, we keep local state as playing
        // unless backend says not playing for a while? 
        // Actually, let's trust backend for time, but be careful about stopping the loop.

        setStore("playback", {
            // Trust backend for isPlaying, UNLESS we are in a transition where we expect to be playing.
            // But if we just paused, we want to trust local state.
            // The issue is: if we pause, local is false. Backend might be true.
            // If we use `isPlaying || ...`, it becomes true.
            // So we should only use backend isPlaying if we haven't explicitly paused.
            // But we don't track "explicitly paused".
            // Let's simplify: Trust backend, but if we are locally playing, keep playing until backend confirms stop?
            // No, opposite. If we locally stopped, we should stop.
            isPlaying: isPlaying,
            currentBar,
            startTime: time
        });

        if (isPlaying) {
            animationFrameId = requestAnimationFrame(updatePlaybackState);
        } else {
            animationFrameId = null;
        }
    } catch (e) {
        console.error("Failed to get playback state:", e);
        animationFrameId = null;
        setStore("playback", "isPlaying", false);
    }
};

export const togglePlayback = async () => {
    try {
        if (store.playback.isPlaying) {
            // Optimistic update
            setStore("playback", "isPlaying", false);
            if (animationFrameId !== null) {
                cancelAnimationFrame(animationFrameId);
                animationFrameId = null;
            }

            await DawService.pause();

            // Do NOT call updatePlaybackState immediately, as backend might still report playing.
            // We trust our local "false" state.
            // We might want to fetch time one last time, but safely.
            // For now, just stop.
        } else {
            // Optimistic update
            setStore("playback", "isPlaying", true);

            await DawService.play();

            if (animationFrameId === null) {
                updatePlaybackState();
            }
        }
    } catch (e) {
        console.error("Failed to toggle playback:", e);
        // Revert on error
        setStore("playback", "isPlaying", !store.playback.isPlaying);
    }
};

export const setBpm = (bpm: number) => {
    setStore("info", "bpm", bpm);
};

export const addTrack = () => {
    const id = store.tracks.length; // Simple auto-increment for now, matching backend if we sync
    const newTrack: Track = {
        id,
        name: `Track ${id + 1}`,
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

export const addClip = async (trackId: number, startBar: number, name: string = "Clip", color: string = "#aec6ff") => {
    const contentId = getOrCreateContentId(name, color);
    const content = store.clipLibrary[contentId];

    // Calculate time and duration
    const bpm = store.info.bpm;
    const timeSigNum = store.info.timeSignature[0];
    const secondsPerBeat = 60 / bpm;
    const secondsPerBar = secondsPerBeat * timeSigNum;

    const startTime = (startBar - 1) * secondsPerBar;
    const duration = 8 * secondsPerBar; // Default 8 bars

    try {
        // Call backend to create clip
        // Use instrument 0 (SimpleSynth) by default for now
        const instrumentIds = [0];
        const id = await DawService.addClip(name, startTime, duration, instrumentIds);

        // If content has notes (from copy/paste), sync them to backend
        if (content.notes.length > 0) {
            await DawService.updateClip(id, { notes: content.notes });
        }

        const newClip: ClipInstance = {
            id,
            trackId,
            clipContentId: contentId,
            startBar,
            lengthBars: 8, // Default 8 bars
        }; setStore("clips", [...store.clips, newClip]);
    } catch (e) {
        console.error("Failed to add clip:", e);
    }
};

export const updateClipNotes = async (contentId: string, notes: Note[]) => {
    // 1. Update local library (Flyweight)
    setStore("clipLibrary", contentId, "notes", notes);

    // 2. Find all instances using this content
    const instances = store.clips.filter(c => c.clipContentId === contentId);

    // 3. Update all backend clips
    await Promise.all(instances.map(instance =>
        DawService.updateClip(instance.id, { notes })
    ));
};

export const renameClipContent = async (contentId: string, newName: string) => {
    // Check uniqueness
    const exists = Object.values(store.clipLibrary).some(c => c.name === newName && c.id !== contentId);
    if (exists) {
        throw new Error("Clip name already exists");
    }

    // 1. Update local library
    setStore("clipLibrary", contentId, "name", newName);

    // 2. Find all instances
    const instances = store.clips.filter(c => c.clipContentId === contentId);

    // 3. Update all backend clips
    await Promise.all(instances.map(instance =>
        DawService.updateClip(instance.id, { name: newName })
    ));
};

export const deleteClip = async (instanceId: number) => {
    try {
        await DawService.removeClip(instanceId);
        setStore("clips", clips => clips.filter(c => c.id !== instanceId));
        if (store.selectedClipId === instanceId) {
            selectClip(null);
        }
    } catch (e) {
        console.error("Failed to delete clip:", e);
    }
};

export const duplicateClip = async (instanceId: number) => {
    const original = store.clips.find(c => c.id === instanceId);
    if (!original) return;

    // Create new instance with SAME content ID (Flyweight)
    // Place it after the original (e.g. +4 bars)
    await addClip(original.trackId, original.startBar + original.lengthBars, store.clipLibrary[original.clipContentId].name, store.clipLibrary[original.clipContentId].color);
};

export const updateClipPosition = (instanceId: number, newStartBar: number) => {
    setStore("clips", (c) => c.id === instanceId, "startBar", newStartBar);
};
