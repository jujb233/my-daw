import { invoke } from '@tauri-apps/api/core'
import { Note } from '../store/types'

export interface BackendClip {
    id: number
    name: string
    start_time: number
    duration: number
    instrument_ids: number[]
    instrument_routes: Record<number, number[]>
    notes: Note[]
}

export const DawService = {
    async addClip(
        name: string,
        startTime: number,
        duration: number,
        instrumentIds: number[] = []
    ): Promise<number> {
        return await invoke('add_clip', {
            name,
            startTime,
            duration,
            instrumentIds,
            instrumentRoutes: null
        })
    },

    async updateClip(id: number, updates: Partial<BackendClip>): Promise<void> {
        // Convert camelCase to snake_case for backend if needed, but tauri usually handles it.
        // However, our backend expects specific Option fields.
        // Let's map explicitly to be safe and clean.
        const args: any = { id }
        if (updates.name !== undefined) args.name = updates.name
        if (updates.start_time !== undefined) args.startTime = updates.start_time
        if (updates.duration !== undefined) args.duration = updates.duration
        if (updates.instrument_ids !== undefined) args.instrumentIds = updates.instrument_ids
        if (updates.instrument_routes !== undefined)
            args.instrumentRoutes = updates.instrument_routes
        if (updates.notes !== undefined) args.notes = updates.notes

        await invoke('update_clip', args)
    },

    async getClip(id: number): Promise<BackendClip> {
        return await invoke('get_clip', { id })
    },

    async removeClip(id: number): Promise<void> {
        await invoke('remove_clip', { id })
    },

    async play(): Promise<void> {
        await invoke('play')
    },

    async pause(): Promise<void> {
        await invoke('pause')
    },

    async stop(): Promise<void> {
        await invoke('stop')
    },

    async seek(position: number): Promise<void> {
        await invoke('seek', { position })
    },

    async getPlaybackState(): Promise<[boolean, number]> {
        return await invoke('get_playback_state')
    }
}
