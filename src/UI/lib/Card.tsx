import { JSX, ParentComponent } from 'solid-js'

interface CardProps {
    title?: string
    className?: string
    headerActions?: JSX.Element
}

export const Card: ParentComponent<CardProps> = props => {
    return (
        <div
            class={`bg-gray-800 rounded-lg border border-gray-700 shadow-lg overflow-hidden flex flex-col ${props.className || ''}`}
        >
            {(props.title || props.headerActions) && (
                <div class='bg-gray-900 px-3 py-2 border-b border-gray-700 flex justify-between items-center'>
                    {props.title && (
                        <h3 class='text-sm font-semibold text-gray-300'>{props.title}</h3>
                    )}
                    <div class='flex items-center gap-2'>{props.headerActions}</div>
                </div>
            )}
            <div class='p-3 flex-1 flex flex-col'>{props.children}</div>
        </div>
    )
}
