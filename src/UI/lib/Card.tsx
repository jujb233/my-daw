import { JSX, ParentComponent } from 'solid-js'

interface CardProps {
    title?: string
    className?: string
    headerActions?: JSX.Element
}

export const Card: ParentComponent<CardProps> = props => {
    return (
        <div
            class={`bg-surface-container border-outline-variant flex flex-col overflow-hidden rounded-lg border shadow-sm ${props.className || ''}`}
        >
            {(props.title || props.headerActions) && (
                <div class='bg-surface-container-high border-outline-variant flex min-h-[40px] items-center justify-between border-b px-3 py-2'>
                    {props.title && (
                        <h3 class='text-on-surface text-sm font-semibold'>{props.title}</h3>
                    )}
                    <div class='ml-auto flex items-center gap-2'>{props.headerActions}</div>
                </div>
            )}
            <div class='flex flex-1 flex-col p-3'>{props.children}</div>
        </div>
    )
}
