import { Component, For, createEffect, onCleanup, createSignal } from 'solid-js'
import {
        mixerTracks,
        fetchMixerTracks,
        addMixerTrack,
        startMetering,
        stopMetering,
        meterLevels
} from '../../store/mixer'
import { MixerStrip } from './MixerStrip'
import { IconButton } from '../../UI/lib/IconButton'
import { t } from '../../i18n'

export const MixerPanel: Component = () => {
        const [isOpen, setIsOpen] = createSignal(false)
        const [width, setWidth] = createSignal(600)
        const [isResizing, setIsResizing] = createSignal(false)

        createEffect(() => {
                fetchMixerTracks()
                startMetering()
                onCleanup(() => stopMetering())
        })

        const startResize = (e: MouseEvent | TouchEvent) => {
                e.preventDefault()
                setIsResizing(true)
                const startX = 'touches' in e ? e.touches[0].clientX : e.clientX
                const startWidth = width()

                const handleMove = (moveEvent: MouseEvent | TouchEvent) => {
                        const currentX = 'touches' in moveEvent ? moveEvent.touches[0].clientX : moveEvent.clientX
                        const newWidth = Math.max(
                                200,
                                Math.min(window.innerWidth - 100, startWidth + (currentX - startX))
                        )
                        setWidth(newWidth)
                }

                const handleUp = () => {
                        setIsResizing(false)
                        window.removeEventListener('mousemove', handleMove)
                        window.removeEventListener('mouseup', handleUp)
                        window.removeEventListener('touchmove', handleMove)
                        window.removeEventListener('touchend', handleUp)
                }

                window.addEventListener('mousemove', handleMove)
                window.addEventListener('mouseup', handleUp)
                window.addEventListener('touchmove', handleMove)
                window.addEventListener('touchend', handleUp)
        }

        return (
                <>
                        {/* Mobile Backdrop */}
                        <div
                                class={`fixed inset-0 z-30 bg-black/50 transition-opacity md:hidden ${isOpen() ? 'pointer-events-auto opacity-100' : 'pointer-events-none opacity-0'}`}
                                onClick={() => setIsOpen(false)}
                        ></div>

                        <div
                                class={`bg-surface border-outline-variant absolute top-0 bottom-0 left-0 z-40 flex h-full shrink-0 flex-col border-r shadow-xl md:relative ${isResizing() ? 'transition-none' : 'transition-all duration-300 ease-in-out'} ${isOpen() ? '' : 'w-0 md:w-12'} `}
                                style={{ width: isOpen() ? `${width()}px` : undefined }}
                        >
                                {/* Toggle Handle - Centered on Right Edge */}
                                <div
                                        class='bg-surface border-outline-variant hover:bg-surface-container-high absolute top-1/2 -right-6 z-50 flex h-24 w-6 -translate-y-1/2 cursor-pointer items-center justify-center rounded-r-lg border border-l-0 shadow-md'
                                        onClick={() => setIsOpen(!isOpen())}
                                        title={isOpen() ? 'Collapse' : 'Expand'}
                                >
                                        <svg
                                                xmlns='http://www.w3.org/2000/svg'
                                                height='24'
                                                viewBox='0 -960 960 960'
                                                width='24'
                                                fill='currentColor'
                                                class={`transition-transform duration-300 ${isOpen() ? 'rotate-180' : ''}`}
                                        >
                                                <path d='M504-480 320-664l56-56 240 240-240 240-56-56 184-184Z' />
                                        </svg>
                                </div>

                                {/* Resize Handle - Wider for touch */}
                                <div
                                        class={`absolute top-0 right-0 bottom-0 z-40 w-4 translate-x-2 cursor-col-resize touch-none ${isResizing() ? 'bg-primary/20' : 'hover:bg-primary/10 bg-transparent'}`}
                                        onMouseDown={startResize}
                                        onTouchStart={startResize}
                                        style={{ display: isOpen() ? 'block' : 'none' }}
                                ></div>

                                {/* Collapsed View: Just a vertical label or icon (Desktop only) */}
                                <div
                                        class={`hidden h-full w-12 flex-col items-center gap-4 pt-4 md:flex ${isOpen() ? '!hidden' : ''}`}
                                >
                                        <div class='text-on-surface flex flex-col items-center gap-1 leading-tight font-medium select-none'>
                                                <span>调</span>
                                                <span>音</span>
                                                <span>台</span>
                                        </div>
                                        <IconButton onClick={() => setIsOpen(true)} variant='standard'>
                                                <svg
                                                        xmlns='http://www.w3.org/2000/svg'
                                                        height='24'
                                                        viewBox='0 -960 960 960'
                                                        width='24'
                                                        fill='currentColor'
                                                >
                                                        <path d='M120-320v-80h280v80H120Zm0-160v-80h440v80H120Zm0-160v-80h440v80H120Zm520 480v-160H480v-80h160v-160h80v160h160v80H720v160h-80Z' />
                                                </svg>
                                        </IconButton>
                                </div>

                                {/* Expanded View */}
                                <div class={`flex flex-1 flex-col overflow-hidden ${isOpen() ? 'flex' : 'hidden'}`}>
                                        <div class='border-outline-variant bg-surface-container flex h-12 shrink-0 items-center justify-between border-b px-4'>
                                                <span class='text-on-surface font-bold'>{t('mixer.console')}</span>
                                                <div class='flex gap-2'>
                                                        <IconButton onClick={addMixerTrack} title={t('mixer.addTrack')}>
                                                                <svg
                                                                        xmlns='http://www.w3.org/2000/svg'
                                                                        height='24'
                                                                        viewBox='0 -960 960 960'
                                                                        width='24'
                                                                        fill='currentColor'
                                                                >
                                                                        <path d='M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z' />
                                                                </svg>
                                                        </IconButton>
                                                </div>
                                        </div>
                                        <div class='flex-1 overflow-x-auto overflow-y-hidden p-2'>
                                                <div class='flex h-full gap-2'>
                                                        <For each={mixerTracks()}>
                                                                {track => (
                                                                        <MixerStrip
                                                                                track={track}
                                                                                level={
                                                                                        track.meter_id
                                                                                                ? meterLevels()[
                                                                                                          track.meter_id
                                                                                                  ] || 0
                                                                                                : 0
                                                                                }
                                                                        />
                                                                )}
                                                        </For>
                                                </div>
                                        </div>
                                </div>
                        </div>
                </>
        )
}
