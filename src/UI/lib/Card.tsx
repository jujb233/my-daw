import { JSX, ParentComponent } from 'solid-js'

interface CardProps {
    title?: string
    className?: string
    headerActions?: JSX.Element
}

export const Card: ParentComponent<CardProps> = props => {
    return (
        <div
            class={`bg-surface-container rounded-lg border border-outline-variant shadow-sm overflow-hidden flex flex-col ${props.className || ''}`}
        >
            {(props.title || props.headerActions) && (
                <div class='bg-surface-container-high px-3 py-2 border-b border-outline-variant flex justify-between items-center min-h-[40px]'>
                    {props.title && (
                        <h3 class='text-sm font-semibold text-on-surface'>{props.title}</h3>
                    )}
                    <div class='flex items-center gap-2 ml-auto'>{props.headerActions}</div>
                </div>
            )}
            <div class='p-3 flex-1 flex flex-col'>{props.children}</div>
        </div>
    )
}
