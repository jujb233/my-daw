import { Component, JSX, splitProps, createSignal } from 'solid-js'

interface SliderProps extends Omit<JSX.InputHTMLAttributes<HTMLInputElement>, 'onChange'> {
        label?: string
        valueDisplay?: string | number
        value?: number
        onChange?: (value: number) => void
        min?: number
        max?: number
        step?: number
}

export const Slider: Component<SliderProps> = props => {
        const [local] = splitProps(props, ['label', 'valueDisplay', 'class', 'value', 'onChange', 'min', 'max', 'step'])
        const [isDragging, setIsDragging] = createSignal(false)

        const min = local.min ?? 0
        const max = local.max ?? 100
        const step = local.step ?? 1
        const value = local.value ?? min

        const percentage = () => ((value - min) / (max - min)) * 100

        const handlePointerDown = (e: PointerEvent) => {
                e.preventDefault()
                const target = e.currentTarget as HTMLElement
                target.setPointerCapture(e.pointerId)
                setIsDragging(true)

                const rect = target.getBoundingClientRect()

                const updateValue = (clientX: number) => {
                        const x = Math.max(0, Math.min(rect.width, clientX - rect.left))
                        let pct = x / rect.width

                        let rawValue = min + pct * (max - min)

                        // Snap to step
                        if (step > 0) {
                                rawValue = Math.round(rawValue / step) * step
                        }

                        // Clamp
                        let newValue = Math.max(min, Math.min(max, rawValue))

                        if (local.onChange) {
                                local.onChange(newValue)
                        }
                }

                // Initial update on click
                updateValue(e.clientX)

                const onPointerMove = (moveEvent: PointerEvent) => {
                        updateValue(moveEvent.clientX)
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

        return (
                <div class={`flex touch-none flex-col gap-2 select-none ${local.class || ''}`}>
                        <div class='flex items-center justify-between px-1'>
                                {local.label && (
                                        <label class='text-on-surface-variant text-xs font-medium'>{local.label}</label>
                                )}
                                {local.valueDisplay !== undefined && (
                                        <span class='text-on-surface-variant bg-surface-container rounded px-1 font-mono text-xs'>
                                                {local.valueDisplay}
                                        </span>
                                )}
                        </div>

                        <div class='relative flex h-8 cursor-ew-resize items-center' onPointerDown={handlePointerDown}>
                                {/* Track Background */}
                                <div class='bg-surface-container-highest h-2 w-full overflow-hidden rounded-full'>
                                        {/* Fill */}
                                        <div
                                                class='bg-primary h-full transition-all duration-75'
                                                style={{ width: `${percentage()}%` }}
                                        />
                                </div>

                                {/* Thumb (Larger for touch) */}
                                <div
                                        class={`absolute flex h-6 w-6 items-center justify-center rounded-full border shadow-sm transition-transform duration-100 ${
                                                isDragging()
                                                        ? 'bg-primary-container border-primary scale-110'
                                                        : 'bg-surface border-outline hover:border-primary'
                                        } `}
                                        style={{
                                                left: `${percentage()}%`,
                                                transform: 'translateX(-50%)'
                                        }}
                                >
                                        <div
                                                class={`h-2 w-2 rounded-full ${isDragging() ? 'bg-primary' : 'bg-on-surface-variant'}`}
                                        />
                                </div>
                        </div>
                </div>
        )
}
