import { Component } from 'solid-js'

interface PianoRollNoteProps {
    note: number
    startPx: number
    widthPx: number
    heightPx: number
    velocity: number
    isSelected?: boolean
    onUpdate?: (newStartPx: number, newWidthPx: number, newNoteVal?: number) => void
    onRemove?: () => void
    onClick?: () => void
}

export const PianoRollNote: Component<PianoRollNoteProps> = props => {
    const handleMouseDown = (e: MouseEvent) => {
        if (e.button !== 0) return
        e.stopPropagation()
        props.onClick?.()

        const startX = e.clientX
        const startY = e.clientY
        const startLeft = props.startPx
        const startNote = props.note

        const onMove = (moveEvent: MouseEvent) => {
            const deltaX = moveEvent.clientX - startX
            const deltaY = moveEvent.clientY - startY

            // Calculate pitch change (negative deltaY means higher pitch)
            // Assuming heightPx is the height of one semitone
            const pitchDelta = Math.round(-deltaY / props.heightPx)
            const newNote = Math.max(0, Math.min(127, startNote + pitchDelta))

            props.onUpdate?.(startLeft + deltaX, props.widthPx, newNote)
        }

        const onUp = () => {
            window.removeEventListener('mousemove', onMove)
            window.removeEventListener('mouseup', onUp)
        }

        window.addEventListener('mousemove', onMove)
        window.addEventListener('mouseup', onUp)
    }

    const handleResizeLeft = (e: MouseEvent) => {
        e.stopPropagation()
        const startX = e.clientX
        const startLeft = props.startPx
        const startWidth = props.widthPx

        const onMove = (moveEvent: MouseEvent) => {
            const delta = moveEvent.clientX - startX
            const newWidth = Math.max(5, startWidth - delta)
            const newLeft = startLeft + (startWidth - newWidth)
            props.onUpdate?.(newLeft, newWidth)
        }

        const onUp = () => {
            window.removeEventListener('mousemove', onMove)
            window.removeEventListener('mouseup', onUp)
        }

        window.addEventListener('mousemove', onMove)
        window.addEventListener('mouseup', onUp)
    }

    const handleResizeRight = (e: MouseEvent) => {
        e.stopPropagation()
        const startX = e.clientX
        const startWidth = props.widthPx

        const onMove = (moveEvent: MouseEvent) => {
            const delta = moveEvent.clientX - startX
            const newWidth = Math.max(5, startWidth + delta)
            props.onUpdate?.(props.startPx, newWidth)
        }

        const onUp = () => {
            window.removeEventListener('mousemove', onMove)
            window.removeEventListener('mouseup', onUp)
        }

        window.addEventListener('mousemove', onMove)
        window.addEventListener('mouseup', onUp)
    }

    return (
        <div
            class={`absolute rounded-sm border border-black/30 overflow-hidden cursor-pointer hover:brightness-110 transition-all ${props.isSelected ? 'ring-1 ring-white' : ''}`}
            style={{
                left: `${props.startPx}px`,
                width: `${props.widthPx}px`,
                height: `${props.heightPx - 1}px`, // -1 for gap
                top: '0px', // Positioned by parent
                'background-color': `rgba(59, 130, 246, ${0.5 + props.velocity * 0.5})`
            }}
            onMouseDown={handleMouseDown}
            onContextMenu={e => {
                e.preventDefault()
                props.onRemove?.()
            }}
        >
            {/* Left Resize Handle */}
            <div
                class='absolute left-0 top-0 bottom-0 w-2 cursor-w-resize hover:bg-white/20 z-10'
                onMouseDown={handleResizeLeft}
            ></div>

            {/* Right Resize Handle */}
            <div
                class='absolute right-0 top-0 bottom-0 w-2 cursor-w-resize hover:bg-white/20 z-10'
                onMouseDown={handleResizeRight}
            ></div>
        </div>
    )
}
