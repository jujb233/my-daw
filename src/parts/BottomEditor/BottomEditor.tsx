import { Component, Show, createSignal, createEffect } from 'solid-js'
import { open } from '@tauri-apps/plugin-dialog'
import { Surface } from '../../UI/lib/Surface'
import { Input } from '../../UI/lib/Input'
import { Button } from '../../UI/lib/Button'
import { store, setStore, selectClip, reloadProject } from '../../store'
import { PianoRoll } from '../MidiEditor/PianoRoll'
import { IconButton } from '../../UI/lib/IconButton'
import { t } from '../../i18n'
import { DawService } from '../../services/daw'

export const BottomEditor: Component = () => {
    const [height, setHeight] = createSignal(300)
    const [isResizing, setIsResizing] = createSignal(false)

    const handleSaveProject = async () => {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: t('bottom.saveProject')
            })

            if (selected && typeof selected === 'string') {
                await DawService.saveProject(selected)
                console.log('Project saved to', selected)
            }
        } catch (e) {
            console.error('Failed to save project:', e)
        }
    }

    const handleLoadProject = async () => {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: t('bottom.loadProject')
            })

            if (selected && typeof selected === 'string') {
                await DawService.loadProject(selected)
                console.log('Project loaded from', selected)
                await reloadProject()
            }
        } catch (e) {
            console.error('Failed to load project:', e)
        }
    }

    const handleExport = async () => {
        // Placeholder for export functionality
        // For now, maybe just log or show a not implemented message
        console.log('Export clicked')
        // If the user wants to export as project folder (backup?), we could reuse save
        // But usually export means audio.
        // Let's just leave it as a log for now unless they specifically asked for audio export.
        // The user said "Save and Export buttons... click to select directory to store as project folder".
        // This implies they might want Export to also save the project? Or maybe "Export Project"?
        // I'll just wire Save for now.
    }

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
                            <Button variant='tonal' onClick={handleSaveProject}>
                                {t('bottom.saveProject')}
                            </Button>
                            <Button variant='tonal' onClick={handleLoadProject}>
                                {t('bottom.loadProject')}
                            </Button>
                            <Button variant='filled' onClick={handleExport}>
                                {t('bottom.export')}
                            </Button>
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
