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
            length: updates.length,
            notes: updates.notes,
            instrumentId: updates.instrumentId
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
        await invoke('add_mixer_track')
    },

    async removeTrack(index: number): Promise<void> {
        await invoke('remove_mixer_track', { index })
    },

    async getActivePlugins(): Promise<any[]> {
        return await invoke('get_active_plugins')
    }
}
