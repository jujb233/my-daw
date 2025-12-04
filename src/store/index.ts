import { createStore } from 'solid-js/store'
import { DawService } from '../services/daw'
import { ProjectData, Clip, Position } from './model'
import { defaultTimeService } from '../services/time'

interface UIState {
    playback: {
        isPlaying: boolean;
        currentPosition: Position;
        startTime: number | null;
    };
    selectedTrackId: number | null;
    selectedClipId: string | null;
    snapInterval: string; // '1/4', '1/8', '1/16', etc.
}

type AppStore = ProjectData & UIState;

const DEFAULT_PROJECT: AppStore = {
    info: {
        name: 'New Project',
        artist: 'User',
        bpm: 120,
        timeSignature: { numerator: 4, denominator: 4 }
    },
    tracks: [
        { id: 0, name: 'Grand Piano', color: '#aec6ff', muted: false, soloed: false },
        { id: 1, name: 'Drums', color: '#ffb4ab', muted: false, soloed: false },
        { id: 2, name: 'Bass', color: '#bfc6dc', muted: false, soloed: false }
    ],
    clips: [],
    instruments: [],
    playback: {
        isPlaying: false,
        currentPosition: { bar: 1, beat: 1, sixteenth: 1, tick: 0, time: 0 },
        startTime: null
    },
    selectedTrackId: null,
    selectedClipId: null,
    snapInterval: '1/16'
}

export const [store, setStore] = createStore<AppStore>(DEFAULT_PROJECT)

// --- Actions ---

export const fetchInstruments = async () => {
    try {
        const instruments = await DawService.getActivePlugins()
        setStore('instruments', instruments)
    } catch (e) {
        console.error('Failed to fetch instruments:', e)
    }
}

export const selectClip = (id: string | null) => {
    setStore('selectedClipId', id)
}

export const selectTrack = (id: number) => {
    setStore('selectedTrackId', id)
}

let animationFrameId: number | null = null
let lastPlayRequestTime = 0

const updatePlaybackState = async () => {
    try {
        const [backendIsPlaying, time] = await DawService.getPlaybackState()

        // Update TimeService with current project settings
        defaultTimeService.setBpm(store.info.bpm);
        defaultTimeService.setTimeSignature(store.info.timeSignature);

        const currentTicks = defaultTimeService.secondsToTicks(time);
        const currentPosition = defaultTimeService.ticksToPosition(currentTicks);

        let isPlaying = backendIsPlaying

        // Fix for race condition where backend hasn't started yet
        if (!isPlaying && store.playback.isPlaying && Date.now() - lastPlayRequestTime < 500) {
            isPlaying = true
        }

        setStore('playback', {
            isPlaying,
            currentPosition,
            startTime: time
        })

        if (isPlaying) {
            animationFrameId = requestAnimationFrame(updatePlaybackState)
        } else {
            animationFrameId = null
        }
    } catch (e) {
        console.error('Failed to get playback state:', e)
        animationFrameId = null
        setStore('playback', 'isPlaying', false)
    }
}

export const togglePlayback = async () => {
    try {
        if (store.playback.isPlaying) {
            // Optimistic update
            setStore('playback', 'isPlaying', false)
            if (animationFrameId !== null) {
                cancelAnimationFrame(animationFrameId)
                animationFrameId = null
            }

            await DawService.pause()
        } else {
            // Optimistic update
            setStore('playback', 'isPlaying', true)
            lastPlayRequestTime = Date.now()

            await DawService.play()

            if (animationFrameId === null) {
                updatePlaybackState()
            }
        }
    } catch (e) {
        console.error('Failed to toggle playback:', e)
    }
}

export const addTrack = async () => {
    try {
        await DawService.addTrack()
        // Optimistic update or fetch
        // For now, we just push a new track to the store to match backend logic
        const newId = store.tracks.length
        setStore('tracks', tracks => [
            ...tracks,
            {
                id: newId,
                name: `Track ${newId + 1}`,
                color: '#aec6ff', // Default color
                muted: false,
                soloed: false
            }
        ])
    } catch (e) {
        console.error('Failed to add track:', e)
    }
}

export const addClip = async (trackId: number, start: Position, length: MusicalLength) => {
    try {
        const name = 'New Clip'
        const id = await DawService.addClip(trackId, name, start, length)
        
        const newClip: Clip = {
            id,
            trackId,
            name,
            color: '#3b82f6',
            start,
            length,
            notes: []
        }
        
        setStore('clips', (clips) => [...clips, newClip])
    } catch (e) {
        console.error('Failed to add clip:', e)
    }
}

export const updateClip = (id: string, update: Partial<Clip>) => {
    setStore('clips', (c) => c.id === id, update)
    // Sync with backend
    DawService.updateClip(id, update).catch(e => console.error("Failed to sync clip update:", e))
}

export const removeClip = (id: string) => {
    setStore('clips', (clips) => clips.filter(c => c.id !== id))
    DawService.removeClip(id).catch(e => console.error("Failed to remove clip:", e))
}

export const removeTrack = async (id: number) => {
    try {
        // Find index
        const index = store.tracks.findIndex(t => t.id === id)
        if (index === -1) return

        await DawService.removeTrack(index)
        setStore('tracks', tracks => tracks.filter(t => t.id !== id))
        
        // Also remove clips on this track? Or keep them?
        // Usually we remove them.
        const clipsToRemove = store.clips.filter(c => c.trackId === id)
        for (const clip of clipsToRemove) {
            removeClip(clip.id)
        }
    } catch (e) {
        console.error('Failed to remove track:', e)
    }
}

export const setSnapInterval = (interval: string) => {
    setStore('snapInterval', interval)
}
