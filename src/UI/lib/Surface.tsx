import { Component, JSX, splitProps } from 'solid-js'

interface SurfaceProps extends JSX.HTMLAttributes<HTMLDivElement> {
        level?: 0 | 1 | 2 | 3 | 4 | 5
}

export const Surface: Component<SurfaceProps> = props => {
        const [local, others] = splitProps(props, ['level', 'class', 'children'])

        const levels = {
                0: 'bg-surface',
                1: 'bg-surface-container-low',
                2: 'bg-surface-container',
                3: 'bg-surface-container-high',
                4: 'bg-surface-container-highest',
                5: 'bg-surface-container-highest' // Material 3 usually stops distinct colors around here or uses elevation overlay
        }

        return (
                <div class={`rounded-xl ${levels[local.level || 1]} ${local.class || ''}`} {...others}>
                        {local.children}
                </div>
        )
}
