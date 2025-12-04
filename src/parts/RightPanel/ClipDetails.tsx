import { Component, Show, onMount, For } from 'solid-js'
import { store, updateClip, selectClip, removeClip, fetchInstruments } from '../../store'
import { DawService } from '../../services/daw'
import { t } from '../../i18n'
import { Input } from '../../UI/lib/Input'
import { Button } from '../../UI/lib/Button'
import { defaultTimeService } from '../../services/time'

export const ClipDetails: Component = () => {
    const clip = () => store.clips.find(c => c.id === store.selectedClipId)

    onMount(() => {
        fetchInstruments()
    })

    const handleRename = (newName: string) => {
        const c = clip()
        if (!c) return
        updateClip(c.id, { name: newName })
    }

    const handleInstrumentChange = (instrumentId: string) => {
        const c = clip()
        if (!c) return
        updateClip(c.id, { instrumentId })
    }

    const handleDelete = async () => {
        const c = clip()
        if (!c) return
        if (confirm(t('clip.deleteConfirm') || 'Delete clip?')) {
            removeClip(c.id)
            selectClip(null)
        }
    }

    return (
        <div class='h-full flex flex-col p-4 gap-4 overflow-y-auto'>
            <Show
                when={clip()}
                fallback={
                    <div class='text-on-surface-variant text-center mt-10'>
                        {t('clip.noSelection')}
                    </div>
                }
            >
                <div class='flex flex-col gap-2'>
                    <h2 class='text-lg font-bold text-on-surface'>{t('clip.details')}</h2>

                    <Input
                        label={t('clip.name')}
                        value={clip()!.name}
                        onChange={e => handleRename(e.currentTarget.value)}
                    />

                    {/* Instrument Selector */}
                    <div class='flex flex-col gap-1'>
                        <span class='text-xs text-on-surface-variant'>Instrument</span>
                        <select
                            class='w-full bg-surface-container-highest border border-outline-variant rounded p-2 text-sm text-on-surface outline-none'
                            value={clip()!.instrumentId || ''}
                            onChange={e => handleInstrumentChange(e.currentTarget.value)}
                        >
                            <option value=''>None</option>
                            <For each={store.instruments}>
                                {inst => <option value={inst.id}>{inst.label || inst.name}</option>}
                            </For>
                        </select>
                    </div>

                    <div class='grid grid-cols-2 gap-2'>
                        <div class='flex flex-col gap-1'>
                            <span class='text-xs text-on-surface-variant'>{t('clip.start')}</span>
                            <span class='text-sm text-on-surface font-mono'>
                                {clip()!.start.bar}.{clip()!.start.beat}.{clip()!.start.sixteenth}
                            </span>
                        </div>
                        <div class='flex flex-col gap-1'>
                            <span class='text-xs text-on-surface-variant'>
                                {t('clip.duration')}
                            </span>
                            <span class='text-sm text-on-surface font-mono'>
                                {defaultTimeService
                                    .ticksToSeconds(clip()!.length.totalTicks)
                                    .toFixed(2)}
                                s
                            </span>
                        </div>
                    </div>

                    <div class='mt-4'>
                        <Button
                            variant='filled'
                            class='w-full bg-error text-on-error'
                            onClick={handleDelete}
                        >
                            {t('clip.delete')}
                        </Button>
                    </div>
                </div>
            </Show>
        </div>
    )
}
