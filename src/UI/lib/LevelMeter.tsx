import { Component } from 'solid-js'

interface LevelMeterProps {
    level: number // 0.0 to 1.0 (or higher for clipping)
    className?: string
}

export const LevelMeter: Component<LevelMeterProps> = props => {
    const heightPercent = () => Math.min(100, Math.max(0, props.level * 100))

    // Gradient stops for meter
    // Green (low) -> Yellow (mid) -> Red (high)
    const gradient = 'linear-gradient(to top, #4caf50 0%, #8bc34a 60%, #ffeb3b 80%, #f44336 100%)'

    return (
        <div
            class={`bg-surface-container-highest rounded-sm overflow-hidden relative border border-outline-variant ${props.className || ''}`}
        >
            {/* Background Grid/Ticks */}
            <div class='absolute inset-0 flex flex-col justify-between py-1 opacity-20 pointer-events-none z-10'>
                <div class='w-full h-[1px] bg-on-surface' />
                <div class='w-full h-[1px] bg-on-surface' />
                <div class='w-full h-[1px] bg-on-surface' />
                <div class='w-full h-[1px] bg-on-surface' />
                <div class='w-full h-[1px] bg-on-surface' />
            </div>

            <div
                class='absolute bottom-0 left-0 right-0 transition-all duration-75 ease-out'
                style={{
                    height: `${heightPercent()}%`,
                    background: gradient
                }}
            />
        </div>
    )
}
