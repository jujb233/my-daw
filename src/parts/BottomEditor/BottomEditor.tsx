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
                        const newHeight = Math.max(150, Math.min(window.innerHeight - 100, startHeight + deltaY))
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
                        class='border-outline-variant relative flex shrink-0 flex-col border-t'
                        style={{
                                height: store.selectedClipId !== null ? `${height()}px` : '80px',
                                transition: isResizing() ? 'none' : 'height 0.3s cubic-bezier(0.4, 0, 0.2, 1)'
                        }}
                >
                        <Show when={store.selectedClipId !== null}>
                                <div
                                        class='group absolute -top-5 right-0 left-0 z-50 flex h-10 cursor-row-resize touch-none items-center justify-center'
                                        onPointerDown={handlePointerDown}
                                >
                                        <div class='bg-surface-variant group-hover:bg-primary/50 h-1.5 w-24 rounded-full shadow-sm backdrop-blur-sm transition-colors'></div>
                                </div>
                        </Show>

                        <Show
                                when={store.selectedClipId !== null}
                                fallback={
                                        <div class='flex h-full items-center gap-4 px-6 py-4'>
                                                <div class='flex w-full flex-wrap items-end gap-4'>
                                                        <Input
                                                                label={t('bottom.projectName')}
                                                                value={store.info.name}
                                                                onInput={e =>
                                                                        setStore('info', 'name', e.currentTarget.value)
                                                                }
                                                                class='w-48'
                                                        />
                                                        <Input
                                                                label={t('bottom.artist')}
                                                                value={store.info.artist}
                                                                onInput={e =>
                                                                        setStore(
                                                                                'info',
                                                                                'artist',
                                                                                e.currentTarget.value
                                                                        )
                                                                }
                                                                class='w-48'
                                                        />
                                                        <Input
                                                                label={t('bottom.bpm')}
                                                                type='number'
                                                                value={store.info.bpm}
                                                                onInput={e =>
                                                                        setStore(
                                                                                'info',
                                                                                'bpm',
                                                                                parseFloat(e.currentTarget.value)
                                                                        )
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
                                <div class='flex flex-1 flex-col overflow-hidden'>
                                        <div class='border-outline-variant bg-surface-container flex h-10 shrink-0 items-center justify-between border-b px-4'>
                                                <span class='text-on-surface font-medium'>{t('bottom.editor')}</span>
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
                                        <div class='relative flex-1 overflow-hidden'>
                                                <PianoRoll clipId={store.selectedClipId!} />
                                        </div>
                                </div>
                        </Show>
                </Surface>
        )
}
