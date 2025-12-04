import { Component, createResource, Show } from 'solid-js'
import { PluginUIRegistry } from '../plugins/registry'
import { getPluginParameters } from '../services/plugin'
import { GenericPluginUI } from './GenericPluginUI'

interface PluginHostProps {
    instId: string
    uniqueId: string
    currentValues: Record<number, number>
}

export const PluginHost: Component<PluginHostProps> = props => {
    const [params] = createResource(() => props.uniqueId, getPluginParameters)

    return (
        <Show
            when={PluginUIRegistry[props.uniqueId]}
            fallback={
                <Show when={params()} fallback={<div>Loading...</div>}>
                    <GenericPluginUI
                        instId={props.instId}
                        params={params()!}
                        currentValues={props.currentValues}
                    />
                </Show>
            }
        >
            {(() => {
                const Comp = PluginUIRegistry[props.uniqueId]
                return <Comp instId={props.instId} params={props.currentValues} />
            })()}
        </Show>
    )
}
