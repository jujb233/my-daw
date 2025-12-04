import { Component, createSignal, Show } from 'solid-js'
import { t } from '../../i18n'

interface GridClipProps {
    name: string
    color?: string
    width: number // in pixels
    left: number // in pixels
    isSelected?: boolean
    onRemove?: () => void
    onCommit?: (newLeft: number, newWidth: number, isCopy: boolean) => void
    onClick?: () => void
}

export const GridClip: Component<GridClipProps> = props => {
    // Local state for dragging to avoid flooding the store/backend
    const [dragState, setDragState] = createSignal<{ left: number; width: number } | null>(null)

    const currentLeft = () => dragState()?.left ?? props.left
    const currentWidth = () => dragState()?.width ?? props.width

    const handleMouseDown = (e: MouseEvent) => {
        if (e.button !== 0) return // Only left click
        e.stopPropagation()
        props.onClick?.()

        const startX = e.clientX
        const startLeft = props.left
        const startWidth = props.width

        // Initialize drag state
        setDragState({ left: startLeft, width: startWidth })

        const onMove = (moveEvent: MouseEvent) => {
            const delta = moveEvent.clientX - startX
            setDragState({
                left: startLeft + delta,
                width: startWidth
            })
        }

        const onUp = (upEvent: MouseEvent) => {
            window.removeEventListener('mousemove', onMove)
            window.removeEventListener('mouseup', onUp)

            const finalState = dragState()
            if (finalState) {
                // Ensure we don't commit if nothing changed (prevents jitter)
                if (finalState.left !== startLeft || finalState.width !== startWidth) {
                    props.onCommit?.(finalState.left, finalState.width, upEvent.altKey)
                }
            }
            setDragState(null)
        }

        window.addEventListener('mousemove', onMove)
        window.addEventListener('mouseup', onUp)
    }

    const handleResizeLeft = (e: MouseEvent) => {
        e.stopPropagation()
        const startX = e.clientX
        const startLeft = props.left
        const startWidth = props.width

        setDragState({ left: startLeft, width: startWidth })

        const onMove = (moveEvent: MouseEvent) => {
            const delta = moveEvent.clientX - startX
            const newWidth = Math.max(10, startWidth - delta)
            const newLeft = startLeft + (startWidth - newWidth)
            setDragState({ left: newLeft, width: newWidth })
        }

        const onUp = (upEvent: MouseEvent) => {
            window.removeEventListener('mousemove', onMove)
            window.removeEventListener('mouseup', onUp)

            const finalState = dragState()
            if (finalState) {
                props.onCommit?.(finalState.left, finalState.width, false) // Resize is never a copy
            }
            setDragState(null)
        }

        window.addEventListener('mousemove', onMove)
        window.addEventListener('mouseup', onUp)
    }

    const handleResizeRight = (e: MouseEvent) => {
        e.stopPropagation()
        const startX = e.clientX
        const startWidth = props.width
        const startLeft = props.left

        setDragState({ left: startLeft, width: startWidth })

        const onMove = (moveEvent: MouseEvent) => {
            const delta = moveEvent.clientX - startX
            const newWidth = Math.max(10, startWidth + delta)
            setDragState({ left: startLeft, width: newWidth })
        }

        const onUp = (upEvent: MouseEvent) => {
            window.removeEventListener('mousemove', onMove)
            window.removeEventListener('mouseup', onUp)

            const finalState = dragState()
            if (finalState) {
                props.onCommit?.(finalState.left, finalState.width, false)
            }
            setDragState(null)
        }

        window.addEventListener('mousemove', onMove)
        window.addEventListener('mouseup', onUp)
    }

    return (
        <div
            class={`absolute h-full rounded border overflow-hidden cursor-pointer hover:brightness-110 transition-all group flex flex-col ${props.isSelected ? 'border-white ring-1 ring-white' : 'border-black/20'} ${dragState() ? 'opacity-80 z-50' : ''}`}
            style={{
                width: `${currentWidth()}px`,
                left: `${currentLeft()}px`,
                'background-color': props.color || '#3b82f6'
            }}
            onMouseDown={handleMouseDown}
            onDblClick={e => e.stopPropagation()}
        >
            {/* Left Resize Handle */}
            <div
                class='absolute left-0 top-0 bottom-0 w-2 cursor-w-resize hover:bg-white/20 z-10'
                onMouseDown={handleResizeLeft}
            ></div>

            <div class='px-2 py-1 text-xs font-medium text-white truncate select-none flex justify-between items-center pointer-events-none'>
                <span>{props.name}</span>
            </div>

            {/* Right Resize Handle */}
            <div
                class='absolute right-0 top-0 bottom-0 w-2 cursor-w-resize hover:bg-white/20 z-10'
                onMouseDown={handleResizeRight}
            ></div>

            {/* Remove Button (visible on hover) */}
            {props.onRemove && (
                <div
                    class='absolute top-1 right-1 opacity-0 group-hover:opacity-100 transition-opacity bg-black/50 rounded-full p-0.5 cursor-pointer hover:bg-black/70 z-20'
                    onMouseDown={e => {
                        e.stopPropagation()
                        props.onRemove?.()
                    }}
                    title={t('sidebar.remove')}
                >
                    <svg
                        xmlns='http://www.w3.org/2000/svg'
                        height='12'
                        viewBox='0 -960 960 960'
                        width='12'
                        fill='white'
                    >
                        <path d='M280-120q-33 0-56.5-23.5T200-200v-520h-40v-80h200v-40h240v40h200v80h-40v520q0 33-23.5 56.5T680-120H280Zm400-600H280v520h400v-520ZM360-280h80v-360h-80v360Zm160 0h80v-360h-80v360ZM280-720v520-520Z' />
                    </svg>
                </div>
            )}
        </div>
    )
}
