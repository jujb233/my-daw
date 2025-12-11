import { Component, createSignal } from 'solid-js'

interface FaderProps {
    value: number // 0.0 to 1.0
    onChange: (val: number) => void
    className?: string
    defaultValue?: number
}

export const Fader: Component<FaderProps> = props => {
    const [isDragging, setIsDragging] = createSignal(false)
    let trackRef: HTMLDivElement | undefined

    const handlePointerDown = (e: PointerEvent) => {
        e.preventDefault()
        e.stopPropagation()
        const target = e.currentTarget as HTMLElement
        target.setPointerCapture(e.pointerId)
        setIsDragging(true)

        const startY = e.clientY
        const startValue = props.value

        // Calculate sensitivity based on track height for 1:1 movement
        const rect = trackRef?.getBoundingClientRect()
        const trackHeight = rect?.height || 200
        const sensitivity = 1 / trackHeight

        const onPointerMove = (moveEvent: PointerEvent) => {
            const deltaY = startY - moveEvent.clientY // Up is positive
            const isFine = moveEvent.shiftKey
            const currentSensitivity = isFine ? sensitivity * 0.1 : sensitivity

            let newValue = startValue + deltaY * currentSensitivity
            newValue = Math.max(0, Math.min(1, newValue))

            props.onChange(newValue)
        }

        const onPointerUp = (upEvent: PointerEvent) => {
            target.releasePointerCapture(upEvent.pointerId)
            setIsDragging(false)
            target.removeEventListener('pointermove', onPointerMove)
            target.removeEventListener('pointerup', onPointerUp)
        }

        target.addEventListener('pointermove', onPointerMove)
        target.addEventListener('pointerup', onPointerUp)
    }

    const handleDblClick = () => {
        props.onChange(props.defaultValue ?? 0.75)
    }

    return (
        <div
            class={`relative flex touch-none justify-center select-none ${props.className || ''}`}
            onDblClick={handleDblClick}
            onPointerDown={handlePointerDown}
        >
            {/* Track Background */}
            <div
                ref={trackRef}
                class='bg-surface-container-highest pointer-events-none relative h-full w-2 overflow-hidden rounded-full'
            >
                {/* Fill Level (Optional, maybe for meters, but faders usually just have a handle) */}
                <div
                    class='bg-primary/20 pointer-events-none absolute right-0 bottom-0 left-0'
                    style={{ height: `${props.value * 100}%` }}
                />
            </div>

            {/* Hit Area & Handle Container */}
            <div class='pointer-events-none absolute inset-0 flex cursor-ns-resize items-end justify-center'>
                {/* Thumb / Handle */}
                <div
                    class={`pointer-events-auto absolute mb-[-24px] flex h-12 w-8 items-center justify-center rounded border shadow-md transition-colors duration-75 ${
                        isDragging()
                            ? 'bg-primary-container border-primary'
                            : 'bg-surface-container-high border-outline hover:border-outline-variant'
                    } `}
                    style={{
                        bottom: `${props.value * 100}%`,
                        transform: 'translateY(50%)'
                    }}
                >
                    {/* Handle Grip Lines */}
                    <div class='flex flex-col gap-0.5 opacity-50'>
                        <div class='bg-on-surface h-[1px] w-4' />
                        <div class='bg-on-surface h-[1px] w-4' />
                        <div class='bg-on-surface h-[1px] w-4' />
                    </div>
                </div>
            </div>
        </div>
    )
}
