import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

export interface PluginInstance {
    id: number; // Index in the backend list
    name: string;
    params: { [key: number]: number }; // Local param ID -> Value
}

export const [instances, setInstances] = createSignal<PluginInstance[]>([]);

export const addInstance = async (name: string) => {
    try {
        await invoke("add_plugin_instance", { name });
        setInstances(prev => [
            ...prev,
            {
                id: prev.length,
                name,
                params: { 0: 0.5, 1: 0 } // Default params for SimpleSynth
            }
        ]);
    } catch (e) {
        console.error("Failed to add plugin instance:", e);
    }
};

export const sendParameter = async (paramId: number, value: number) => {
    try {
        await invoke("update_parameter", { paramId, value });
    } catch (e) {
        console.error("Failed to update parameter:", e);
    }
};

export const updateInstanceParam = (instanceId: number, paramId: number, value: number) => {
    // Update local state
    setInstances(prev => prev.map(inst => {
        if (inst.id === instanceId) {
            return { ...inst, params: { ...inst.params, [paramId]: value } };
        }
        return inst;
    }));

    // Calculate global param ID: instanceId * 2 + paramId
    // NOTE: This assumes all plugins have exactly 2 params. 
    // In a real app, we'd query the plugin for its param count or offset.
    const globalId = instanceId * 2 + paramId;
    sendParameter(globalId, value);
};
