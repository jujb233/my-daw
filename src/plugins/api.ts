import { invoke } from '@tauri-apps/api/core'

export enum PluginType {
        Native = 'Native',
        Clap = 'Clap',
        Vst = 'Vst'
}

export type ParameterType = 'Float' | 'Int' | 'Bool' | { Enum: string[] }

export interface PluginParameter {
        id: number
        name: string
        min_value: number
        max_value: number
        default_value: number
        value_type: ParameterType
}

export interface PluginInfo {
        name: string
        vendor: string
        url: string
        plugin_type: PluginType
        unique_id: string
}

export async function listAvailablePlugins(): Promise<PluginInfo[]> {
        return (await invoke('get_available_plugins')) as PluginInfo[]
}

export async function rescanPlugins(): Promise<PluginInfo[]> {
        return (await invoke('rescan_plugins')) as PluginInfo[]
}

export async function scanProjectPlugins(projectPath: string): Promise<PluginInfo[]> {
        return (await invoke('scan_project_plugins', { projectPath })) as PluginInfo[]
}

export async function getPluginParameters(uniqueId: string): Promise<PluginParameter[] | null> {
        return (await invoke('get_plugin_parameters', { uniqueId })) as PluginParameter[] | null
}

export async function setParameter(paramId: number, value: number): Promise<void> {
        await invoke('update_parameter', { param_id: paramId, value })
}

export async function addPluginInstance(name: string): Promise<void> {
        await invoke('add_plugin_instance', { name })
}

export async function getActivePlugins(): Promise<any[]> {
        return (await invoke('get_active_plugins')) as any[]
}

export async function getInstanceParameters(
        instanceId: string
): Promise<{ params: PluginParameter[]; values: number[] } | null> {
        return (await invoke('get_instance_parameters', { instanceId })) as {
                params: PluginParameter[]
                values: number[]
        } | null
}

export async function setInstanceParameter(instanceId: string, paramId: number, value: number): Promise<void> {
        await invoke('set_instance_parameter', { instanceId, param_id: paramId, value })
}
