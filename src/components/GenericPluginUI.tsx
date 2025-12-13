import { Component } from 'solid-js'
import { PluginParameter } from '../plugins/api'

interface GenericPluginUIProps {
    instId: string
    params: PluginParameter[]
    currentValues: Record<number, number>}
export const GenericPluginUI: Component<GenericPluginUIProps> = _props => {
    return <div class='p-2 text-sm text-on-surface-variant'>Embedded plugin UI not included.</div>

    
}
