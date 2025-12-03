import { Component, For } from 'solid-js'
import { Button } from '../../UI/lib/Button'
import {
    instances,
    removeInstance,
    updateInstanceLabel,
    toggleInstanceExpanded
} from '../../store/audio'
import { openSettings } from '../../store/ui'
import { t } from '../../i18n'
import { InstrumentCard } from '../../UI/components/InstrumentCard'
import { PluginHost } from '../../components/PluginHost'

export const InstrumentList: Component = () => {
    return (
        <div class='flex-1 overflow-y-auto p-4 flex flex-col gap-3'>
            <For each={instances()}>
                {inst => (
                    <InstrumentCard
                        label={inst.label}
                        name={inst.name}
                        id={inst.id}
                        isExpanded={inst.isExpanded}
                        onToggleExpand={() => toggleInstanceExpanded(inst.id)}
                        onRemove={() => removeInstance(inst.id)}
                    >
                        {/* Label Editor */}
                        <div>
                            <label class='text-xs text-on-surface-variant block mb-1'>
                                {t('sidebar.label')}
                            </label>
                            <input
                                type='text'
                                value={inst.label}
                                onInput={e => updateInstanceLabel(inst.id, e.currentTarget.value)}
                                class='w-full bg-surface-container-highest text-on-surface text-sm px-2 py-1 rounded border-none focus:ring-1 focus:ring-primary outline-none'
                            />
                        </div>

                        {/* Plugin UI */}
                        <PluginHost
                            instId={inst.id}
                            uniqueId={inst.name}
                            currentValues={inst.params}
                        />
                    </InstrumentCard>
                )}
            </For>

            <div class='mt-2'>
                <Button variant='outlined' class='w-full' onClick={() => openSettings('plugins')}>
                    {t('sidebar.addTimbre')}
                </Button>
            </div>
        </div>
    )
}
