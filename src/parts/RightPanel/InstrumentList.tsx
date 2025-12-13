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
import { GenericPluginUI } from '../../components/GenericPluginUI'

export const InstrumentList: Component = () => {
    return (
        <div class='flex flex-1 flex-col gap-3 overflow-y-auto p-4'>
            <For each={instances()}>
                {inst => (
                    <InstrumentCard
                        label={inst.label}
                        name={
                            t(`plugins.${inst.name}`) === `plugins.${inst.name}`
                                ? inst.name
                                : t(`plugins.${inst.name}`)
                        }
                        id={inst.id}
                        isExpanded={inst.isExpanded}
                        onToggleExpand={() => toggleInstanceExpanded(inst.id)}
                        onRemove={() => removeInstance(inst.index)}
                    >
                        {/* Label Editor */}
                        <div>
                            <label class='text-on-surface-variant mb-1 block text-xs'>
                                {t('sidebar.label')}
                            </label>
                            <input
                                type='text'
                                value={inst.label}
                                onInput={e =>
                                    updateInstanceLabel(inst.index, e.currentTarget.value)
                                }
                                class='bg-surface-container-highest text-on-surface focus:ring-primary w-full rounded border-none px-2 py-1 text-sm outline-none focus:ring-1'
                            />
                        </div>

                        {/* Plugin UI: render built-in/native plugin UIs when available */}
                        <div class='text-on-surface-variant text-sm'>
                            <GenericPluginUI
                                uniqueId={inst.name}
                                instanceId={inst.id}
                                currentValues={inst.params}
                            />
                        </div>
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
