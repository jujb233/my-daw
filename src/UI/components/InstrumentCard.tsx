import { Component, JSX, Show } from 'solid-js'
import { Card } from '../lib/Card'
import { IconButton } from '../lib/IconButton'

interface InstrumentCardProps {
    label: string
    name: string
    id: number
    isExpanded?: boolean
    onToggleExpand?: () => void
    onRemove?: () => void
    children?: JSX.Element
}

export const InstrumentCard: Component<InstrumentCardProps> = props => {
    return (
        <Card
            className='bg-surface-container-low border-outline-variant'
            headerActions={
                <div class='flex items-center gap-1'>
                    {props.onRemove && (
                        <IconButton
                            variant='standard'
                            class='text-on-surface-variant hover:text-error w-8 h-8'
                            onClick={e => {
                                e.stopPropagation()
                                props.onRemove?.()
                            }}
                        >
                            <svg
                                xmlns='http://www.w3.org/2000/svg'
                                height='20'
                                viewBox='0 -960 960 960'
                                width='20'
                                fill='currentColor'
                            >
                                <path d='M280-120q-33 0-56.5-23.5T200-200v-520h-40v-80h200v-40h240v40h200v80h-40v520q0 33-23.5 56.5T680-120H280Zm400-600H280v520h400v-520ZM360-280h80v-360h-80v360Zm160 0h80v-360h-80v360ZM280-720v520-520Z' />
                            </svg>
                        </IconButton>
                    )}
                    <IconButton
                        variant='standard'
                        class={`text-on-surface-variant transition-transform duration-200 w-8 h-8 ${props.isExpanded ? 'rotate-180' : ''}`}
                        onClick={e => {
                            e.stopPropagation()
                            props.onToggleExpand?.()
                        }}
                    >
                        <svg
                            xmlns='http://www.w3.org/2000/svg'
                            height='24'
                            viewBox='0 -960 960 960'
                            width='24'
                            fill='currentColor'
                        >
                            <path d='M480-345 240-585l56-56 184 184 184-184 56 56-240 240Z' />
                        </svg>
                    </IconButton>
                </div>
            }
        >
            {/* Header Content (Always Visible) */}
            <div class='flex items-center gap-3 cursor-pointer' onClick={props.onToggleExpand}>
                <div class='w-10 h-10 rounded bg-primary/20 flex items-center justify-center text-primary shrink-0'>
                    🎹
                </div>
                <div class='flex flex-col flex-1 min-w-0'>
                    <span class='text-sm font-medium text-on-surface truncate'>{props.label}</span>
                    <span class='text-xs text-on-surface-variant truncate'>
                        {props.name} #{props.id + 1}
                    </span>
                </div>
            </div>

            {/* Expanded Content */}
            <Show when={props.isExpanded}>
                <div class='pt-3 mt-3 border-t border-outline-variant/50 flex flex-col gap-3'>
                    {props.children}
                </div>
            </Show>
        </Card>
    )
}
