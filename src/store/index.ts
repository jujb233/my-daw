import { createStore } from 'solid-js/store'
import { DawService } from '../services/daw'
import { ProjectData, Clip, Position, MusicalLength } from './model'
import { defaultTimeService, SnapGrid } from '../services/time'

interface UIState {
    playback: {
        isPlaying: boolean
        currentPosition: Position
        startTime: number | null
    }
    selectedTrackId: number | null
    selectedClipId: string | null
    snapInterval: SnapGrid // '1/4', '1/8', '1/16', etc.
}

type AppStore = ProjectData & UIState

const DEFAULT_PROJECT: AppStore = {
    info: {
        name: 'New Project',
        artist: 'User',
        bpm: 120,
        timeSignature: { numerator: 4, denominator: 4 }
    },
    tracks: [
        {
            id: 0,
            name: 'Track 1',
            color: '#aec6ff',
            muted: false,
            soloed: false,
            targetMixerTrackId: 1
        },
        {
            id: 1,
            name: 'Track 2',
            color: '#ffb4ab',
            muted: false,
            soloed: false,
            targetMixerTrackId: 2
        },
        {
            id: 2,
            name: 'Track 3',
            color: '#bfc6dc',
            muted: false,
            soloed: false,
            targetMixerTrackId: 3
        },
        {
            id: 3,
            name: 'Track 4',
            color: '#bfc6dc',
            muted: false,
            soloed: false,
            targetMixerTrackId: 4
        }
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
        defaultTimeService.setBpm(store.info.bpm)
        defaultTimeService.setTimeSignature(store.info.timeSignature)

        const currentTicks = defaultTimeService.secondsToTicks(time)
        const currentPosition = defaultTimeService.ticksToPosition(currentTicks)

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
        await fetchTracks()
    } catch (e) {
        console.error('Failed to add track:', e)
    }
}

export const fetchTracks = async () => {
    try {
        const tracks = await DawService.getTracks()
        setStore('tracks', tracks)
    } catch (e) {
        console.error('Failed to fetch tracks:', e)
    }
}

export const fetchClips = async () => {
    try {
        const clips = await DawService.getAllClips()
        setStore('clips', clips)
    } catch (e) {
        console.error('Failed to fetch clips:', e)
    }
}

export const reloadProject = async () => {
    await fetchTracks()
    await fetchClips()
    // Also reload mixer tracks if needed, but they might be part of tracks or separate
    // For now tracks and clips are the main arrangement data
}

export const addClip = async (trackId: number, start: Position, length: MusicalLength) => {
    try {
        // Auto-name: Clip 1, Clip 2...
        const existingNames = new Set(store.clips.map(c => c.name))
        let i = 1
        while (existingNames.has(`Clip ${i}`)) i++
        const name = `Clip ${i}`

        const id = await DawService.addClip(trackId, name, start, length)

        const newClip: Clip = {
            id,
            trackId,
            name,
            color: '#3b82f6',
            start,
            length,
            notes: [],
            content: { type: 'Midi' },
            instrumentIds: [],
            instrumentRoutes: {}
        }

        setStore('clips', clips => [...clips, newClip])
    } catch (e) {
        console.error('Failed to add clip:', e)
    }
}

export const copyClip = async (originalId: string, newTrackId: number, newStart: Position) => {
    try {
        const original = store.clips.find(c => c.id === originalId)
        if (!original) return

        const id = await DawService.copyClip(originalId, newTrackId, newStart)

        const newClip: Clip = {
            ...original,
            id,
            trackId: newTrackId,
            start: newStart
            // Name, notes, length, instruments are shared/copied
        }

        setStore('clips', clips => [...clips, newClip])
    } catch (e) {
        console.error('Failed to copy clip:', e)
    }
}

export const updateClip = (id: string, update: Partial<Clip>) => {
    // 1. Determine if this is a content update or instance update
    const isContentUpdate =
        update.notes !== undefined ||
        update.length !== undefined ||
        update.instrumentIds !== undefined ||
        update.instrumentRoutes !== undefined ||
        update.name !== undefined

    if (isContentUpdate) {
        // Find the clip to get its name
        const target = store.clips.find(c => c.id === id)
        if (target) {
            // Update ALL clips with same name
            setStore(
                'clips',
                c => c.name === target.name,
                clip => {
                    // Only update content properties, preserve instance properties unless specifically targeting this instance
                    const newClip = { ...clip, ...update }

                    // If this is NOT the target instance, revert instance-specific properties
                    if (clip.id !== id) {
                        if (update.start) newClip.start = clip.start
                        if (update.trackId) newClip.trackId = clip.trackId
                    }
                    return newClip
                }
            )
        }
    } else {
        // Instance update (start, trackId)
        setStore('clips', c => c.id === id, update)
    }

    // Sync with backend
    if (update.instrumentIds) {
        DawService.log(
            `Store: updateClip calling backend with instrumentIds: ${JSON.stringify(update.instrumentIds)}`
        )
    }
    DawService.updateClip(id, update).catch(e => console.error('Failed to sync clip update:', e))
}

export const removeClip = (id: string) => {
    setStore('clips', clips => clips.filter(c => c.id !== id))
    DawService.removeClip(id).catch(e => console.error('Failed to remove clip:', e))
}

export const removeTrack = async (id: number) => {
    try {
        await DawService.removeTrack(id)
        await fetchTracks()

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
export const setSnapInterval = (interval: SnapGrid) => {
    setStore('snapInterval', interval)
}
