import { Component } from 'solid-js'

interface PluginHostProps {
    instId: string
    uniqueId: string
    currentValues: Record<number, number>
}

export const PluginHost: Component<PluginHostProps> = _props => {
    return (
        <div class='text-on-surface-variant text-sm'>External plugin UI not bundled with host.</div>
    )
}
