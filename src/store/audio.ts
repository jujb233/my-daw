import { createSignal } from 'solid-js'
import { invoke } from '@tauri-apps/api/core'

export interface PluginInstance {
    id: number // Index in the backend list
    name: string
    label: string
    params: { [key: number]: number } // Local param ID -> Value
    isExpanded?: boolean
    routingTrackId: number
}

export const [instances, setInstances] = createSignal<PluginInstance[]>([])

export const addInstance = async (name: string) => {
    try {
        await invoke('add_plugin_instance', { name })
        setInstances(prev => [
            ...prev,
            {
                id: prev.length,
                name,
                label: 'New Instrument',
                params: { 10: 0.5, 11: 0 }, // Default params for SimpleSynth (10=Gain, 11=Wave)
                isExpanded: true,
                routingTrackId: 0
            }
        ])
    } catch (e) {
        console.error('Failed to add plugin instance:', e)
    }
}

export const removeInstance = async (index: number) => {
    try {
        await invoke('remove_plugin_instance', { index })
        // Re-index remaining instances
        setInstances(prev => {
            const filtered = prev.filter(inst => inst.id !== index)
            return filtered.map((inst, i) => ({ ...inst, id: i }))
        })
    } catch (e) {
        console.error('Failed to remove plugin instance:', e)
    }
}

export const updateInstanceLabel = async (index: number, label: string) => {
    try {
        await invoke('update_plugin_label', { index, label })
        setInstances(prev => prev.map(inst => (inst.id === index ? { ...inst, label } : inst)))
    } catch (e) {
        console.error('Failed to update plugin label:', e)
    }
}

export const updateInstanceRouting = async (index: number, trackId: number) => {
    try {
        await invoke('set_instrument_routing', { instIndex: index, trackIndex: trackId })
        setInstances(prev =>
            prev.map(inst => (inst.id === index ? { ...inst, routingTrackId: trackId } : inst))
        )
    } catch (e) {
        console.error('Failed to update plugin routing:', e)
    }
}

export const toggleInstanceExpanded = (index: number) => {
    setInstances(prev =>
        prev.map(inst => (inst.id === index ? { ...inst, isExpanded: !inst.isExpanded } : inst))
    )
}

export const sendParameter = async (paramId: number, value: number) => {
    try {
        await invoke('update_parameter', { paramId, value })
    } catch (e) {
        console.error('Failed to update parameter:', e)
    }
}

export const updateInstanceParam = (instanceId: number, paramId: number, value: number) => {
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
    const globalId = 10000 + instanceId * 100 + paramId
    sendParameter(globalId, value)
}
