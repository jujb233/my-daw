import { Component, Show, createSignal, createEffect } from 'solid-js'
import { Surface } from '../../UI/lib/Surface'
import { Input } from '../../UI/lib/Input'
import { Button } from '../../UI/lib/Button'
import { store, setStore, selectClip } from '../../store'
import { PianoRoll } from '../MidiEditor/PianoRoll'
import { IconButton } from '../../UI/lib/IconButton'
import { t } from '../../i18n'

export const BottomEditor: Component = () => {
    const [height, setHeight] = createSignal(300)
    const [isResizing, setIsResizing] = createSignal(false)

    createEffect(() => {
        if (store.selectedClipId !== null && height() < 150) {
            setHeight(300)
        }
    })

    const handlePointerDown = (e: PointerEvent) => {
        if (e.button !== 0 && e.pointerType === 'mouse') return
        e.preventDefault()
        e.stopPropagation()

        const target = e.currentTarget as HTMLElement
        target.setPointerCapture(e.pointerId)
        setIsResizing(true)

        const startY = e.clientY
        const startHeight = height()

        const handlePointerMove = (moveEvent: PointerEvent) => {
            const deltaY = startY - moveEvent.clientY
            const newHeight = Math.max(
                150,
                Math.min(window.innerHeight - 100, startHeight + deltaY)
            )
            setHeight(newHeight)
        }

        const handlePointerUp = (upEvent: PointerEvent) => {
            setIsResizing(false)
            target.releasePointerCapture(upEvent.pointerId)
            target.removeEventListener('pointermove', handlePointerMove)
            target.removeEventListener('pointerup', handlePointerUp)
        }

        target.addEventListener('pointermove', handlePointerMove)
        target.addEventListener('pointerup', handlePointerUp)
    }

    return (
        <Surface
            level={2}
            class='flex flex-col shrink-0 border-t border-outline-variant relative'
            style={{
                height: store.selectedClipId !== null ? `${height()}px` : '80px',
                transition: isResizing() ? 'none' : 'height 0.3s cubic-bezier(0.4, 0, 0.2, 1)'
            }}
        >
            <Show when={store.selectedClipId !== null}>
                <div
                    class='absolute -top-5 left-0 right-0 h-10 cursor-row-resize z-50 flex items-center justify-center group touch-none'
                    onPointerDown={handlePointerDown}
                >
                    <div class='w-24 h-1.5 rounded-full bg-surface-variant group-hover:bg-primary/50 transition-colors shadow-sm backdrop-blur-sm'></div>
                </div>
            </Show>

            <Show
                when={store.selectedClipId !== null}
                fallback={
                    <div class='flex items-center px-6 py-4 gap-4 h-full'>
                        <div class='flex gap-4 flex-wrap items-end w-full'>
                            <Input
                                label={t('bottom.projectName')}
                                value={store.info.name}
                                onInput={e => setStore('info', 'name', e.currentTarget.value)}
                                class='w-48'
                            />
                            <Input
                                label={t('bottom.artist')}
                                value={store.info.artist}
                                onInput={e => setStore('info', 'artist', e.currentTarget.value)}
                                class='w-48'
                            />
                            <Input
                                label={t('bottom.bpm')}
                                type='number'
                                value={store.info.bpm}
                                onInput={e =>
                                    setStore('info', 'bpm', parseFloat(e.currentTarget.value))
                                }
                                class='w-24'
                            />
                            <Input
                                label={t('bottom.timeSig')}
                                value={`${store.info.timeSignature.numerator}/${store.info.timeSignature.denominator}`}
                                class='w-24'
                                disabled
                            />
                            <div class='flex-grow'></div>
                            <Button variant='tonal'>{t('bottom.saveProject')}</Button>
                            <Button variant='filled'>{t('bottom.export')}</Button>
                        </div>
                    </div>
                }
            >
                <div class='flex-1 flex flex-col overflow-hidden'>
                    <div class='h-10 border-b border-outline-variant flex items-center justify-between px-4 bg-surface-container shrink-0'>
                        <span class='font-medium text-on-surface'>{t('bottom.editor')}</span>
                        <IconButton onClick={() => selectClip(null)} variant='standard'>
                            <svg
                                xmlns='http://www.w3.org/2000/svg'
                                height='20'
                                viewBox='0 -960 960 960'
                                width='20'
                                fill='currentColor'
                            >
                                <path d='m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z' />
                            </svg>
                        </IconButton>
                    </div>
                    <div class='flex-1 overflow-hidden relative'>
                        <PianoRoll clipId={store.selectedClipId!} />
                    </div>
                </div>
            </Show>
        </Surface>
    )
}
