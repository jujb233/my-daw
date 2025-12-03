import { Component, For, Match, Switch } from 'solid-js'
import { PluginParameter } from '../plugins/api'
import { Slider } from '../UI/lib/Slider'
import { updateInstanceParam } from '../store/audio'

interface GenericPluginUIProps {
    instId: number
    params: PluginParameter[]
    currentValues: Record<number, number>
}

export const GenericPluginUI: Component<GenericPluginUIProps> = props => {
    return (
        <div class='flex flex-col gap-2 p-2'>
            <For each={props.params}>
                {param => (
                    <div>
                        <Switch>
                            <Match
                                when={
                                    typeof param.value_type === 'object' &&
                                    'Enum' in param.value_type
                                }
                            >
                                <div class='flex flex-col gap-1'>
                                    <span class='text-xs text-on-surface-variant'>
                                        {param.name}
                                    </span>
                                    <div class='flex gap-1'>
                                        <For each={(param.value_type as { Enum: string[] }).Enum}>
                                            {(option, index) => (
                                                <button
                                                    class={`px-2 py-1 text-xs rounded ${
                                                        (props.currentValues[param.id] ??
                                                            param.default_value) === index()
                                                            ? 'bg-primary text-on-primary'
                                                            : 'bg-surface-container-highest'
                                                    }`}
                                                    onClick={() =>
                                                        updateInstanceParam(
                                                            props.instId,
                                                            param.id,
                                                            index()
                                                        )
                                                    }
                                                >
                                                    {option}
                                                </button>
                                            )}
                                        </For>
                                    </div>
                                </div>
                            </Match>
                            <Match when={true}>
                                <Slider
                                    label={param.name}
                                    min={param.min_value}
                                    max={param.max_value}
                                    step={param.value_type === 'Int' ? 1 : 0.01}
                                    value={props.currentValues[param.id] ?? param.default_value}
                                    onChange={val =>
                                        updateInstanceParam(props.instId, param.id, val)
                                    }
                                    valueDisplay={`${(props.currentValues[param.id] ?? param.default_value).toFixed(2)}`}
                                />
                            </Match>
                        </Switch>
                    </div>
                )}
            </For>
        </div>
    )
}
