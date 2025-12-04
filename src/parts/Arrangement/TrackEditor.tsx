import { Component, For, Show } from 'solid-js'
import {
    store,
    selectTrack,
    selectClip,
    updateClip,
    addTrack,
    addClip,
    removeTrack,
    copyClip
} from '../../store'
import { t } from '../../i18n'
import { Button } from '../../UI/lib/Button'
import { invoke } from '@tauri-apps/api/core'
import { defaultTimeService, PPQ, SnapGrid } from '../../services/time'
import { GridClip } from './GridClip'
import { MusicalLength } from '../../store/model'

const PIXELS_PER_BAR = 60

const Playhead: Component = () => {
    // Offset by 200px (Header width)
    // We need to calculate pixels based on currentPosition
    const ticks = () => defaultTimeService.positionToTicks(store.playback.currentPosition)
    const ticksPerBar = () => PPQ * store.info.timeSignature.numerator
    const left = () => (ticks() / ticksPerBar()) * PIXELS_PER_BAR + 200

    return (
        <div
            class='absolute top-0 bottom-0 w-[1px] bg-red-500 z-50 pointer-events-none'
            style={{ left: `${left()}px` }}
        >
            <div class='absolute -top-1 -left-1.5 w-3 h-3 bg-red-500 rotate-45'></div>
        </div>
    )
}

import { setStore } from '../../store'

const Ruler: Component<{ scrollRef: (el: HTMLDivElement) => void }> = props => {
    const handleMouseDown = (e: MouseEvent) => {
        const target = e.currentTarget as HTMLDivElement
        const rect = target.getBoundingClientRect()

        const updatePosition = (clientX: number) => {
            const x = clientX - rect.left
            const bar = x / PIXELS_PER_BAR + 1
            const ticks = (bar - 1) * (PPQ * store.info.timeSignature.numerator)
            const time = defaultTimeService.ticksToSeconds(ticks)

            // Optimistic update
            const newPos = defaultTimeService.ticksToPosition(ticks)
            setStore('playback', 'currentPosition', newPos)

            invoke('seek', { position: time })
        }

        updatePosition(e.clientX)

        const onMove = (moveEvent: MouseEvent) => {
            updatePosition(moveEvent.clientX)
        }

        const onUp = () => {
            window.removeEventListener('mousemove', onMove)
            window.removeEventListener('mouseup', onUp)
        }

        window.addEventListener('mousemove', onMove)
        window.addEventListener('mouseup', onUp)
    }

    return (
        <div class='h-8 bg-surface-container-highest border-b border-outline-variant flex items-end z-20 shrink-0'>
            <div class='w-[200px] shrink-0 border-r border-outline-variant bg-surface-container-highest flex items-center justify-center text-xs text-on-surface font-bold'>
                {t('tracks.header')}
            </div>
            <div
                ref={props.scrollRef}
                class='flex-1 relative overflow-hidden whitespace-nowrap cursor-pointer h-full'
            >
                {/* Container for ruler content that matches track width */}
                <div class='h-full relative min-w-[4000px]' onMouseDown={handleMouseDown}>
                    <div class='absolute bottom-0 left-0 w-full h-full flex text-xs text-on-surface-variant font-mono pointer-events-none'>
                        <For each={Array.from({ length: 100 })}>
                            {(_, i) => {
                                const barNum = i() + 1
                                return (
                                    <div
                                        class='absolute bottom-0 border-l border-outline-variant pl-1 h-4 flex items-center select-none'
                                        style={{ left: `${(barNum - 1) * PIXELS_PER_BAR}px` }}
                                    >
                                        {barNum}
                                    </div>
                                )
                            }}
                        </For>
                    </div>
                    {/* Playhead Marker in Ruler */}
                    <div
                        class='absolute bottom-0 h-full w-[1px] bg-red-500 z-50 pointer-events-none'
                        style={{
                            left: `${(defaultTimeService.positionToTicks(store.playback.currentPosition) / (PPQ * store.info.timeSignature.numerator)) * PIXELS_PER_BAR}px`
                        }}
                    >
                        <div class='absolute top-0 -left-1.5 w-3 h-3 bg-red-500 rotate-45'></div>
                    </div>
                </div>
            </div>
        </div>
    )
}

const TrackHeader: Component<{ track: any }> = props => {
    return (
        <div
            class={`w-[200px] shrink-0 h-24 border-r border-b border-outline-variant flex flex-col p-2 gap-2 relative group cursor-pointer transition-colors ${store.selectedTrackId === props.track.id ? 'bg-secondary-container/30' : 'bg-surface-container-low'}`}
            onClick={() => selectTrack(props.track.id)}
        >
            <div
                class='absolute left-0 top-0 bottom-0 w-1'
                style={{ 'background-color': props.track.color }}
            ></div>
            <div class='flex justify-between items-center pl-2'>
                <span class='font-medium text-sm truncate text-on-surface'>{props.track.name}</span>
                <div class='flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity'>
                    <div
                        title={t('icons.mute')}
                        class={`w-4 h-4 rounded text-[10px] flex items-center justify-center cursor-pointer ${props.track.muted ? 'bg-primary text-on-primary' : 'bg-on-surface-variant/20 hover:bg-primary hover:text-on-primary'}`}
                    >
                        M
                    </div>
                    <div
                        title={t('icons.solo')}
                        class={`w-4 h-4 rounded text-[10px] flex items-center justify-center cursor-pointer ${props.track.soloed ? 'bg-tertiary text-on-tertiary' : 'bg-on-surface-variant/20 hover:bg-tertiary hover:text-on-tertiary'}`}
                    >
                        S
                    </div>
                    <div
                        title={t('tracks.delete')}
                        class='w-4 h-4 rounded text-[10px] flex items-center justify-center cursor-pointer bg-on-surface-variant/20 hover:bg-error hover:text-on-error'
                        onClick={e => {
                            e.stopPropagation()
                            if (confirm(t('tracks.deleteConfirm') || 'Delete track?')) {
                                removeTrack(props.track.id)
                            }
                        }}
                    >
                        X
                    </div>
                </div>
            </div>
        </div>
    )
}

const TrackLane: Component<{ track: any }> = props => {
    const pixelsPerTick = () => {
        const ticksPerBar = PPQ * store.info.timeSignature.numerator
        return PIXELS_PER_BAR / ticksPerBar
    }

    const handleClipCommit = (
        clipId: string,
        newLeftPx: number,
        newWidthPx: number,
        isCopy: boolean,
        isResize: boolean
    ) => {
        const ppt = pixelsPerTick()

        // Convert pixels to ticks
        let startTicks = newLeftPx / ppt
        let durationTicks = newWidthPx / ppt

        // Snap
        const snap = (store.snapInterval || '1/16') as SnapGrid
        startTicks = defaultTimeService.snapTicks(startTicks, snap)
        durationTicks = defaultTimeService.snapTicks(durationTicks, snap)

        // Ensure non-negative
        startTicks = Math.max(0, startTicks)
        durationTicks = Math.max(PPQ / 4, durationTicks)

        const newStart = defaultTimeService.ticksToPosition(startTicks)

        const length: MusicalLength = {
            bars: 0,
            beats: 0,
            sixteenths: 0,
            ticks: 0,
            totalTicks: durationTicks,
            seconds: defaultTimeService.ticksToSeconds(durationTicks)
        }

        if (isCopy) {
            copyClip(clipId, props.track.id, newStart)
        } else {
            // Only update length if it actually changed (resizing), otherwise keep original length
            // But wait, handleClipCommit receives newWidthPx.
            // If we are just moving, newWidthPx should be the same as before.
            // However, we are recalculating 'length' from 'durationTicks'.
            // This might lose precision or reset some properties if not careful.
            // But for now, let's trust the calculation.

            // IMPORTANT: We must also update the trackId if the clip was moved to a different track!
            // But GridClip is inside a For loop filtered by trackId.
            // Dragging across tracks is not yet supported by this simple drag handler
            // because the handler is bound to the specific TrackLane.
            // To support cross-track dragging, we'd need a global drag state.
            // For now, let's assume movement is within the same track (horizontal).

            if (isResize) {
                updateClip(clipId, {
                    start: newStart,
                    length: length
                })
            } else {
                updateClip(clipId, {
                    start: newStart
                })
            }
        }
    }

    const handleDblClick = (e: MouseEvent) => {
        // Ignore clicks on clips or other children if we want strictly empty space
        // But e.target check is tricky if there are grid lines.
        // Grid lines have pointer-events-none so they are fine.
        // Clips stop propagation? Or we check target.

        // If we click on a clip, the clip's onClick/onDblClick handles it.
        // But here we are on the lane div.

        const rect = (e.currentTarget as HTMLDivElement).getBoundingClientRect()
        const x = e.clientX - rect.left

        const ppt = pixelsPerTick()
        let startTicks = x / ppt

        // Snap to beat (1/4) for new clips
        const snap = (store.snapInterval || '1/4') as SnapGrid
        startTicks = defaultTimeService.snapTicks(startTicks, snap)

        const start = defaultTimeService.ticksToPosition(startTicks)

        // Default length 1 bar
        const lengthTicks = PPQ * store.info.timeSignature.numerator
        const length: MusicalLength = {
            bars: 1,
            beats: 0,
            sixteenths: 0,
            ticks: 0,
            totalTicks: lengthTicks,
            seconds: defaultTimeService.ticksToSeconds(lengthTicks)
        }

        addClip(props.track.id, start, length)
    }

    return (
        <div
            class='flex-1 h-24 border-b border-outline-variant relative bg-surface-container-lowest min-w-[4000px]'
            onDblClick={handleDblClick}
        >
            {/* Grid Lines */}
            <div class='absolute inset-0 pointer-events-none'>
                <For each={Array.from({ length: 100 })}>
                    {(_, i) => (
                        <div
                            class='absolute top-0 bottom-0 border-l border-outline-variant/30'
                            style={{ left: `${i() * PIXELS_PER_BAR}px` }}
                        ></div>
                    )}
                </For>
            </div>

            {/* Clips */}
            <For each={store.clips}>
                {clip => (
                    <Show when={clip.trackId === props.track.id}>
                        <GridClip
                            name={clip.name}
                            color={clip.color}
                            width={clip.length.totalTicks * pixelsPerTick()}
                            left={defaultTimeService.positionToTicks(clip.start) * pixelsPerTick()}
                            isSelected={store.selectedClipId === clip.id}
                            onClick={() => selectClip(clip.id)}
                            onCommit={(l, w, isCopy, isResize) =>
                                handleClipCommit(clip.id, l, w, isCopy, isResize)
                            }
                        />
                    </Show>
                )}
            </For>
        </div>
    )
}

export const TrackEditor: Component = () => {
    let scrollContainer: HTMLDivElement | undefined
    let rulerScroll: HTMLDivElement | undefined

    const handleScroll = (e: Event) => {
        const target = e.target as HTMLDivElement
        if (rulerScroll) {
            rulerScroll.scrollLeft = target.scrollLeft
        }
    }

    return (
        <div class='flex-1 flex flex-col overflow-hidden bg-surface'>
            <Ruler scrollRef={el => (rulerScroll = el)} />
            <div class='flex-1 overflow-y-auto overflow-x-hidden relative'>
                <Playhead />
                <div
                    ref={scrollContainer}
                    class='absolute inset-0 overflow-auto'
                    onScroll={handleScroll}
                >
                    <div class='min-w-max'>
                        <For each={store.tracks}>
                            {track => (
                                <div class='flex'>
                                    <TrackHeader track={track} />
                                    <TrackLane track={track} />
                                </div>
                            )}
                        </For>
                        {/* Add Track Button Area */}
                        <div class='flex'>
                            <div class='w-[200px] shrink-0 p-2 border-r border-outline-variant'>
                                <Button
                                    variant='text'
                                    class='w-full justify-start'
                                    onClick={() => {
                                        addTrack()
                                    }}
                                >
                                    + {t('tracks.add')}
                                </Button>
                            </div>
                            <div class='flex-1 border-b border-outline-variant bg-surface-container-lowest'></div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    )
}
