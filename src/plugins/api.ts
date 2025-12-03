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
