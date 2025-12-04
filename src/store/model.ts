export interface TimeSignature {
    numerator: number;
    denominator: number;
}

export interface Position {
    bar: number;      // 1-based measure index
    beat: number;     // 1-based beat index
    sixteenth: number;// 1-based sixteenth index
    tick: number;     // 0-based tick (0-239 assuming 240 PPQ for 16th, or standard 960 PPQ)
    
    // The "Time" part of the user's request
    time: number;     // Absolute time in seconds
}

export interface MusicalLength {
    bars: number;
    beats: number;
    sixteenths: number;
    ticks: number;
    
    totalTicks: number; // Helper for calculations
    seconds: number;    // Helper for audio engine
}

export interface Note {
    id: string;
    note: number; // MIDI pitch
    start: Position;
    duration: MusicalLength;
    velocity: number;
    selected?: boolean;
}

export interface Clip {
    id: string;
    trackId: number;
    name: string;
    color: string;
    start: Position;
    length: MusicalLength;
    notes: Note[];
    instrumentIds: string[];
    instrumentRoutes: Record<string, number>; // InstrumentID -> TrackID
    isSelected?: boolean;
}

export interface Track {
    id: number;
    name: string;
    color: string;
    muted: boolean;
    soloed: boolean;
    // ... other fields
}

export interface PluginInstanceData {
    id: string;
    name: string;
    label: string;
    routing_track_index: number;
}

export interface ProjectData {
    info: {
        name: string;
        artist: string;
        bpm: number;
        timeSignature: TimeSignature;
    };
    tracks: Track[];
    clips: Clip[];
    instruments: PluginInstanceData[];
}
