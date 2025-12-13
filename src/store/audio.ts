import { createSignal } from 'solid-js'
import { invoke } from '@tauri-apps/api/core'
import { DawService } from '../services/daw'

export interface PluginInstance {
        id: string // UUID from backend
        index: number // Array index for backend commands
        name: string
        label: string
        params: { [key: number]: number } // Local param ID -> Value
        isExpanded?: boolean
        routingTrackId: number
}

export const [instances, setInstances] = createSignal<PluginInstance[]>([])

export const fetchInstances = async () => {
        try {
                const plugins = await DawService.getActivePlugins()
                setInstances(prev => {
                        return plugins.map((p: any, idx: number) => {
                                const existing = prev.find(i => i.id === p.id)
                                return {
                                        id: p.id,
                                        index: idx,
                                        name: p.name,
                                        label: p.label,
                                        routingTrackId: p.routing_track_index,
                                        isExpanded: existing?.isExpanded ?? true,
                                        params: existing?.params ?? { 10: 0.5, 11: 0 } // Default params
                                }
                        })
                })
        } catch (e) {
                console.error('Failed to fetch plugin instances:', e)
        }
}

export const addInstance = async (name: string) => {
        try {
                await invoke('add_plugin_instance', { name })
                await fetchInstances()
        } catch (e) {
                console.error('Failed to add plugin instance:', e)
        }
}

export const removeInstance = async (index: number) => {
        try {
                await invoke('remove_plugin_instance', { index })
                await fetchInstances()
        } catch (e) {
                console.error('Failed to remove plugin instance:', e)
        }
}

export const updateInstanceLabel = async (index: number, label: string) => {
        try {
                await invoke('update_plugin_label', { index, label })
                await fetchInstances()
        } catch (e) {
                console.error('Failed to update plugin label:', e)
        }
}

export const updateInstanceRouting = async (index: number, trackId: number) => {
        try {
                await invoke('set_instrument_routing', { instIndex: index, trackIndex: trackId })
                await fetchInstances()
        } catch (e) {
                console.error('Failed to update plugin routing:', e)
        }
}

export const toggleInstanceExpanded = (id: string) => {
        setInstances(prev => prev.map(inst => (inst.id === id ? { ...inst, isExpanded: !inst.isExpanded } : inst)))
}

export const updateInstanceParams = (id: string, paramId: number, value: number) => {
        setInstances(prev =>
                prev.map(inst => (inst.id === id ? { ...inst, params: { ...inst.params, [paramId]: value } } : inst))
        )
}

export const sendParameter = async (paramId: number, value: number) => {
        try {
                await invoke('update_parameter', { paramId, value })
        } catch (e) {
                console.error('Failed to update parameter:', e)
        }
}

export const updateInstanceParam = (instanceId: string, paramId: number, value: number) => {
        // Update local state
        setInstances(prev =>
                prev.map(inst => {
                        if (inst.id === instanceId) {
                                return { ...inst, params: { ...inst.params, [paramId]: value } }
                        }
                        return inst
                })
        )

        // Calculate global param ID: 10000 + instanceId * 100 + paramId
        // Note: This logic needs to be updated if instanceId is UUID.
        // For now, we need the numeric index for the global ID calculation if the backend expects it.
        // But wait, the backend `update_parameter` takes `param_id: u32`.
        // If we are using UUIDs, we can't easily map to a u32 ID unless we look up the index.

        // Let's find the instance to get its index.
        const inst = instances().find(i => i.id === instanceId)
        if (inst) {
                const globalId = 10000 + inst.index * 100 + paramId
                sendParameter(globalId, value)
        }
}
