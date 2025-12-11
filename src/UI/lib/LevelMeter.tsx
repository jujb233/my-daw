import { Component, For, mergeProps } from 'solid-js'

interface LevelMeterProps {
    level: number // 0.0 to 1.0 (or higher for clipping)
    className?: string
    showLabels?: boolean
}

export const LevelMeter: Component<LevelMeterProps> = _props => {
    const props = mergeProps({ showLabels: true }, _props)

    const heightPercent = () => Math.min(100, Math.max(0, props.level * 100))

    // Gradient stops for meter
    // Green (low) -> Yellow (mid) -> Red (high)
    const gradient = 'linear-gradient(to top, #4caf50 0%, #8bc34a 60%, #ffeb3b 80%, #f44336 100%)'

    const ticks = [
        { db: 0, percent: 100, label: '0' },
        { db: -6, percent: 50, label: '-6' },
        { db: -12, percent: 25, label: '-12' },
        { db: -24, percent: 6.3, label: '-24' }
    ]

    return (
        <div class={`flex gap-1 ${props.className || ''}`}>
            <div class='bg-surface-container-highest border-outline-variant relative h-full min-w-[10px] flex-1 overflow-hidden rounded-sm border'>
                {/* Background Grid/Ticks */}
                <div class='pointer-events-none absolute inset-0 z-10'>
                    <For each={ticks}>
                        {tick => (
                            <div
                                class='bg-on-surface absolute h-[1px] w-full opacity-40'
                                style={{ bottom: `${tick.percent}%` }}
                            />
                        )}
                    </For>
                </div>

                <div
                    class='absolute right-0 bottom-0 left-0 transition-all duration-75 ease-out'
                    style={{
                        height: `${heightPercent()}%`,
                        background: gradient
                    }}
                />
            </div>

            {props.showLabels && (
                <div class='text-on-surface-variant relative h-full w-6 font-mono text-[9px] select-none'>
                    <For each={ticks}>
                        {tick => (
                            <div
                                class='absolute w-full text-left leading-none'
                                style={{
                                    bottom: `${tick.percent}%`,
                                    transform: 'translateY(50%)'
                                }}
                            >
                                {tick.label}
                            </div>
                        )}
                    </For>
                    <div
                        class='absolute w-full text-left leading-none'
                        style={{ bottom: '0%', transform: 'translateY(50%)' }}
                    >
                        -∞
                    </div>
                </div>
            )}
        </div>
    )
}
