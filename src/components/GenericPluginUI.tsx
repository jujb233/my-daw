import { Component, For, createSignal, onMount } from 'solid-js'
import { listen } from '@tauri-apps/api/event'
import type { PluginParameter } from '../plugins/api'
import { getInstanceParameters, setInstanceParameter } from '../plugins/api'

interface GenericPluginUIProps {
        uniqueId: string
        instanceId: string
        currentValues: Record<number, number>
}

export const GenericPluginUI: Component<GenericPluginUIProps> = props => {
        const [params, setParams] = createSignal<PluginParameter[]>([])
        const [values, setValues] = createSignal<number[]>([])

        onMount(async () => {
                try {
                        const res = await getInstanceParameters(props.instanceId)
                        if (res) {
                                setParams(res.params)
                                setValues(res.values)
                        }
                } catch (e) {
                        // ignore
                }

                // listen for parameter change events and update UI values
                try {
                        const unlisten = await listen('plugin-parameter-changed', event => {
                                // payload: { instanceId, paramId, value }
                                const payload: any = event.payload as any
                                if (payload.instanceId === props.instanceId) {
                                        const pId = payload.paramId as number
                                        const val = payload.value as number
                                        const idx = params().findIndex(p => p.id === pId)
                                        if (idx >= 0) {
                                                const next = [...values()]
                                                next[idx] = val
                                                setValues(next)
                                        }
                                }
                        })
                        // we do not unlisten on unmount for simplicity; listener is lightweight
                        void unlisten
                } catch (e) {
                        // ignore if event subsystem unavailable
                }
        })

        const onChange = async (index: number, id: number, v: number) => {
                try {
                        await setInstanceParameter(props.instanceId, id, v)
                        const next = [...values()]
                        next[index] = v
                        setValues(next)
                } catch (e) {
                        console.error('Failed to set parameter', e)
                }
        }

        return (
                <div class='text-on-surface-variant space-y-2 p-2 text-sm'>
                        <For each={params()}>
                                {(param, i) => (
                                        <div class='flex items-center gap-3'>
                                                <div class='text-on-surface-variant w-28 text-xs'>{param.name}</div>
                                                <input
                                                        type='range'
                                                        min={param.min_value}
                                                        max={param.max_value}
                                                        step={(param.max_value - param.min_value) / 100}
                                                        value={values()[i()] ?? param.default_value}
                                                        onInput={e =>
                                                                onChange(
                                                                        i(),
                                                                        param.id,
                                                                        parseFloat(e.currentTarget.value)
                                                                )
                                                        }
                                                        class='flex-1'
                                                />
                                                <div class='w-12 text-right text-xs'>
                                                        {(values()[i()] ?? param.default_value).toFixed(2)}
                                                </div>
                                        </div>
                                )}
                        </For>
                        {params().length === 0 && (
                                <div class='text-on-surface-variant text-sm'>No parameters exposed.</div>
                        )}
                </div>
        )
}
