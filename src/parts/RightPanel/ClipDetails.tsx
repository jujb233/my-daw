import { Component, Show, onMount, For } from 'solid-js'
import { store, updateClip, selectClip, removeClip } from '../../store'
import { instances, fetchInstances } from '../../store/audio'
import { t } from '../../i18n'
import { Input } from '../../UI/lib/Input'
import { Button } from '../../UI/lib/Button'
import { defaultTimeService } from '../../services/time'
import { DawService } from '../../services/daw'

export const ClipDetails: Component = () => {
    const clip = () => store.clips.find(c => c.id === store.selectedClipId)

    onMount(() => {
        fetchInstances()
    })

    const handleRename = (newName: string) => {
        const c = clip()
        if (!c) return
        updateClip(c.id, { name: newName })
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
        <div class='flex h-full flex-col gap-4 overflow-y-auto p-4'>
            <Show
                when={clip()}
                fallback={
                    <div class='text-on-surface-variant mt-10 text-center'>
                        {t('clip.noSelection')}
                    </div>
                }
            >
                <div class='flex flex-col gap-2'>
                    <h2 class='text-on-surface text-lg font-bold'>{t('clip.details')}</h2>

                    <Input
                        label={t('clip.name')}
                        value={clip()!.name}
                        onChange={e => handleRename(e.currentTarget.value)}
                    />

                    {/* Instrument Selector & Routing */}
                    <div class='flex flex-col gap-2'>
                        <span class='text-on-surface-variant text-xs'>
                            {t('clip.instrumentsAndRouting') || 'Instruments & Routing'}
                        </span>
                        <div class='border-outline-variant bg-surface-container-low flex max-h-[200px] flex-col gap-2 overflow-y-auto rounded border p-2'>
                            <Show
                                when={instances().length > 0}
                                fallback={
                                    <div class='text-on-surface-variant py-4 text-center text-xs'>
                                        {t('clip.noInstruments') || 'No instruments available'}
                                    </div>
                                }
                            >
                                <For each={instances()}>
                                    {inst => {
                                        const isSelected = () =>
                                            clip()!.instrumentIds?.includes(inst.id)
                                        return (
                                            <div class='border-outline-variant/50 flex flex-col gap-1 border-b pb-2 last:border-0'>
                                                <div class='flex items-center gap-2'>
                                                    <input
                                                        type='checkbox'
                                                        checked={isSelected()}
                                                        onChange={e => {
                                                            const c = clip()!
                                                            let newIds = [
                                                                ...(c.instrumentIds || [])
                                                            ]
                                                            if (e.currentTarget.checked) {
                                                                newIds.push(inst.id)
                                                            } else {
                                                                newIds = newIds.filter(
                                                                    id => id !== inst.id
                                                                )
                                                            }
                                                            console.log(
                                                                'ClipDetails: Toggling instrument',
                                                                inst.id,
                                                                'New IDs:',
                                                                newIds
                                                            )
                                                            DawService.log(
                                                                `ClipDetails: Toggling instrument ${inst.id}. New IDs: ${JSON.stringify(newIds)}`
                                                            )
                                                            // Force update by creating a new array reference
                                                            updateClip(c.id, {
                                                                instrumentIds: [...newIds]
                                                            })
                                                        }}
                                                    />
                                                    <span class='text-on-surface flex-1 truncate text-sm'>
                                                        {inst.label || inst.name}
                                                    </span>
                                                </div>

                                                <Show when={isSelected()}>
                                                    <div class='flex items-center gap-2 pl-6'>
                                                        <span class='text-on-surface-variant text-xs'>
                                                            Route to:
                                                        </span>
                                                        <select
                                                            class='bg-surface-container-highest border-outline-variant text-on-surface flex-1 rounded border p-1 text-xs outline-none'
                                                            value={
                                                                clip()!.instrumentRoutes?.[
                                                                    inst.id
                                                                ] ?? clip()!.trackId
                                                            }
                                                            onChange={e => {
                                                                const c = clip()!
                                                                const trackId = parseInt(
                                                                    e.currentTarget.value
                                                                )
                                                                const newRoutes = {
                                                                    ...(c.instrumentRoutes || {})
                                                                }
                                                                newRoutes[inst.id] = trackId
                                                                updateClip(c.id, {
                                                                    instrumentRoutes: newRoutes
                                                                })
                                                            }}
                                                        >
                                                            <For each={store.tracks}>
                                                                {track => (
                                                                    <option value={track.id}>
                                                                        {track.name}
                                                                    </option>
                                                                )}
                                                            </For>
                                                        </select>
                                                    </div>
                                                </Show>
                                            </div>
                                        )
                                    }}
                                </For>
                            </Show>
                        </div>
                    </div>

                    <div class='grid grid-cols-2 gap-2'>
                        <div class='flex flex-col gap-1'>
                            <span class='text-on-surface-variant text-xs'>{t('clip.start')}</span>
                            <span class='text-on-surface font-mono text-sm'>
                                {clip()!.start.bar}.{clip()!.start.beat}.{clip()!.start.sixteenth}
                            </span>
                        </div>
                        <div class='flex flex-col gap-1'>
                            <span class='text-on-surface-variant text-xs'>
                                {t('clip.duration')}
                            </span>
                            <span class='text-on-surface font-mono text-sm'>
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
                            class='bg-error text-on-error w-full'
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
