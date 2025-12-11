import { Component, JSX, Show } from 'solid-js'
import { Card } from '../lib/Card'
import { IconButton } from '../lib/IconButton'

interface InstrumentCardProps {
    label: string
    name: string
    id: string
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
                            class='text-on-surface-variant hover:text-error h-8 w-8'
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
                        class={`text-on-surface-variant h-8 w-8 transition-transform duration-200 ${props.isExpanded ? 'rotate-180' : ''}`}
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
            <div class='flex cursor-pointer items-center gap-3' onClick={props.onToggleExpand}>
                <div class='bg-primary/20 text-primary flex h-10 w-10 shrink-0 items-center justify-center rounded'>
                    🎹
                </div>
                <div class='flex min-w-0 flex-1 flex-col'>
                    <span class='text-on-surface truncate text-sm font-medium'>{props.label}</span>
                    <span class='text-on-surface-variant truncate text-xs'>
                        {props.name} #{props.id + 1}
                    </span>
                </div>
            </div>

            {/* Expanded Content */}
            <Show when={props.isExpanded}>
                <div class='border-outline-variant/50 mt-3 flex flex-col gap-3 border-t pt-3'>
                    {props.children}
                </div>
            </Show>
        </Card>
    )
}
