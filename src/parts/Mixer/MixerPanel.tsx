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
            const currentX =
                'touches' in moveEvent ? moveEvent.touches[0].clientX : moveEvent.clientX
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
                class={`fixed inset-0 bg-black/50 z-30 md:hidden transition-opacity ${isOpen() ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}`}
                onClick={() => setIsOpen(false)}
            ></div>

            <div
                class={`
                    absolute left-0 top-0 bottom-0 md:relative 
                    z-40 bg-surface border-r border-outline-variant 
                    flex flex-col shadow-xl h-full shrink-0
                    ${isResizing() ? 'transition-none' : 'transition-all duration-300 ease-in-out'}
                    ${isOpen() ? '' : 'w-0 md:w-12'}
                `}
                style={{ width: isOpen() ? `${width()}px` : undefined }}
            >
                {/* Toggle Handle - Centered on Right Edge */}
                <div
                    class='absolute -right-6 top-1/2 -translate-y-1/2 w-6 h-24 bg-surface border border-l-0 border-outline-variant rounded-r-lg cursor-pointer flex items-center justify-center hover:bg-surface-container-high z-50 shadow-md'
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
                    class={`absolute right-0 top-0 bottom-0 w-4 translate-x-2 cursor-col-resize z-40 touch-none ${isResizing() ? 'bg-primary/20' : 'bg-transparent hover:bg-primary/10'}`}
                    onMouseDown={startResize}
                    onTouchStart={startResize}
                    style={{ display: isOpen() ? 'block' : 'none' }}
                ></div>

                {/* Collapsed View: Just a vertical label or icon (Desktop only) */}
                <div
                    class={`h-full w-12 flex-col items-center pt-4 gap-4 hidden md:flex ${isOpen() ? '!hidden' : ''}`}
                >
                    <div class='flex flex-col items-center text-on-surface font-medium select-none leading-tight gap-1'>
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
                <div class={`flex-1 flex flex-col overflow-hidden ${isOpen() ? 'flex' : 'hidden'}`}>
                    <div class='h-12 shrink-0 flex items-center justify-between px-4 border-b border-outline-variant bg-surface-container'>
                        <span class='font-bold text-on-surface'>{t('mixer.console')}</span>
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
                        <div class='flex gap-2 h-full'>
                            <For each={mixerTracks()}>
                                {track => (
                                    <MixerStrip
                                        track={track}
                                        level={
                                            track.meter_id ? meterLevels()[track.meter_id] || 0 : 0
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
