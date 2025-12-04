import { invoke } from '@tauri-apps/api/core'
import { Clip, Position, MusicalLength } from '../store/model'

export const DawService = {
    async addClip(
        trackId: number,
        name: string,
        start: Position,
        length: MusicalLength
    ): Promise<string> {
        return await invoke('add_clip', {
            trackId,
            name,
            start,
            length
        })
    },

    async updateClip(id: string, updates: Partial<Clip>): Promise<void> {
        await invoke('update_clip', {
            id,
            name: updates.name,
            start: updates.start,
            track_id: updates.trackId,
            length: updates.length,
            notes: updates.notes,
            instrument_ids: updates.instrumentIds,
            instrument_routes: updates.instrumentRoutes
        })
    },

    async copyClip(originalId: string, newTrackId: number, newStart: Position): Promise<string> {
        return await invoke('copy_clip', {
            originalId,
            newTrackId,
            newStart
        })
    },

    async getClip(id: string): Promise<Clip> {
        return await invoke('get_clip', { id })
    },

    async removeClip(id: string): Promise<void> {
        await invoke('remove_clip', { id })
    },

    async play(): Promise<void> {
        await invoke('play')
    },

    async pause(): Promise<void> {
        await invoke('pause')
    },

    async getPlaybackState(): Promise<[boolean, number]> {
        return await invoke('get_playback_state')
    },

    async addTrack(): Promise<void> {
        await invoke('add_arrangement_track')
    },

    async removeTrack(id: number): Promise<void> {
        await invoke('remove_arrangement_track', { id })
    },

    async getTracks(): Promise<any[]> {
        return await invoke('get_arrangement_tracks')
    },

    async getActivePlugins(): Promise<any[]> {
        return await invoke('get_active_plugins')
    }
}
