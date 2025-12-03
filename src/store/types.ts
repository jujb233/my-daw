export interface Note {
    note: number
    relative_start: number
    duration: number
    velocity: number
}

export interface ClipContent {
    id: string
    name: string
    color: string
    notes: Note[]
}

export interface ClipInstance {
    id: number
    trackId: number
    clipContentId: string
    startBar: number
    lengthBars: number
}

export interface Track {
    id: number
    name: string
    color: string
    timbreId?: string
    muted: boolean
    soloed: boolean
}

export interface ProjectInfo {
    name: string
    artist: string
    bpm: number
    timeSignature: [number, number]
}

export interface PlaybackState {
    isPlaying: boolean
    currentBar: number
    startTime: number | null // Timestamp when playback started
}

export interface ProjectStore {
    info: ProjectInfo
    playback: PlaybackState
    tracks: Track[]
    clips: ClipInstance[]
    clipLibrary: Record<string, ClipContent> // id -> Content
    selectedTrackId: number | null
    selectedClipId: number | null
}
