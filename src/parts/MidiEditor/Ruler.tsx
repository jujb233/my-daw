import { Component, For } from 'solid-js'

interface RulerProps {
        zoom: number // pixels per unit
        length: number // total units
        height?: number
        start?: number // start unit
        onClick?: (value: number) => void
}

export const Ruler: Component<RulerProps> = props => {
        const markers = () => {
                const count = Math.ceil(props.length)
                return Array.from({ length: count + 1 }, (_, i) => (props.start || 0) + i)
        }

        return (
                <div
                        class='bg-surface-container-high border-outline-variant relative cursor-pointer overflow-hidden border-b select-none'
                        style={{ height: `${props.height || 32}px`, width: `${props.length * props.zoom}px` }}
                        onClick={e => {
                                const rect = e.currentTarget.getBoundingClientRect()
                                const x = e.clientX - rect.left
                                const value = x / props.zoom + (props.start || 0)
                                props.onClick?.(value)
                        }}
                >
                        <For each={markers()}>
                                {i => (
                                        <div
                                                class='border-on-surface-variant/50 text-on-surface-variant absolute bottom-0 flex items-end border-l pb-1 pl-1 text-[10px]'
                                                style={{
                                                        left: `${(i - (props.start || 0)) * props.zoom}px`,
                                                        height: '50%'
                                                }}
                                        >
                                                {i}
                                        </div>
                                )}
                        </For>
                </div>
        )
}
