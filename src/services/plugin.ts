import { invoke } from '@tauri-apps/api/core'
import { PluginInfo, PluginParameter } from '../plugins/api'

export async function getAvailablePlugins(): Promise<PluginInfo[]> {
        return await invoke('get_available_plugins')
}

export async function getPluginParameters(uniqueId: string): Promise<PluginParameter[] | null> {
        return await invoke('get_plugin_parameters', { uniqueId })
}

export async function importPlugin(path: string): Promise<PluginInfo> {
        return await invoke('import_plugin', { path })
}
