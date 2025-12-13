import { Component } from 'solid-js'
import { PluginParameter } from '../plugins/api'

interface GenericPluginUIProps {
    instId: string
    params: PluginParameter[]
    currentValues: Record<number, number>
}
export const GenericPluginUI: Component<GenericPluginUIProps> = _props => {
    return <div class='text-on-surface-variant p-2 text-sm'>Embedded plugin UI not included.</div>
}
