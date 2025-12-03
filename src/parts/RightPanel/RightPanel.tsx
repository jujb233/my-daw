import { Component, Show, createSignal } from 'solid-js'
import { IconButton } from '../../UI/lib/IconButton'
import { t } from '../../i18n'
import { store } from '../../store'
import { InstrumentList } from './InstrumentList'
import { ClipDetails } from './ClipDetails'

export const RightPanel: Component<{ isOpen: boolean; onClose: () => void }> = props => {
    const [width, setWidth] = createSignal(320)
    const [isResizing, setIsResizing] = createSignal(false)

    const startResize = (e: MouseEvent | TouchEvent) => {
        e.preventDefault()
        setIsResizing(true)
        const startX = 'touches' in e ? e.touches[0].clientX : e.clientX
        const startWidth = width()

        const handleMove = (moveEvent: MouseEvent | TouchEvent) => {
            const currentX =
                'touches' in moveEvent ? moveEvent.touches[0].clientX : moveEvent.clientX
            // Dragging left increases width
            const newWidth = Math.max(
                250,
                Math.min(window.innerWidth - 100, startWidth + (startX - currentX))
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
        <div
            class={`relative transition-all duration-300 ease-in-out overflow-hidden flex flex-col border-l border-outline-variant bg-surface-container-low shrink-0 ${props.isOpen ? 'opacity-100' : 'w-0 opacity-0'}`}
            style={{ width: props.isOpen ? `${width()}px` : '0px' }}
        >
            {/* Resize Handle */}
            <div
                class={`absolute left-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-primary z-40 ${isResizing() ? 'bg-primary' : 'bg-transparent'}`}
                onMouseDown={startResize}
                onTouchStart={startResize}
                style={{ display: props.isOpen ? 'block' : 'none' }}
            ></div>

            <div class='h-14 flex items-center justify-between px-4 border-b border-outline-variant shrink-0'>
                <span class='font-medium text-on-surface select-none'>
                    {store.selectedClipId !== null ? t('sidebar.clipDetails') : t('sidebar.title')}
                </span>
                <IconButton onClick={props.onClose} variant='standard'>
                    <svg
                        xmlns='http://www.w3.org/2000/svg'
                        height='24'
                        viewBox='0 -960 960 960'
                        width='24'
                        fill='currentColor'
                    >
                        <path d='m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z' />
                    </svg>
                </IconButton>
            </div>

            <Show when={store.selectedClipId !== null} fallback={<InstrumentList />}>
                <ClipDetails />
            </Show>
        </div>
    )
}
