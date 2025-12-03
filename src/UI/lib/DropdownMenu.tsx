import { Component, createSignal, Show, For, onCleanup, onMount } from 'solid-js'
import { Portal } from 'solid-js/web'

export interface MenuItem {
    label: string
    icon?: any
    onClick?: () => void
    children?: MenuItem[]
}

interface DropdownMenuProps {
    trigger: (props: { onClick: (e: MouseEvent) => void; isOpen: boolean }) => any
    items: MenuItem[]
    class?: string
}

export const DropdownMenu: Component<DropdownMenuProps> = props => {
    const [isOpen, setIsOpen] = createSignal(false)
    const [position, setPosition] = createSignal({ x: 0, y: 0 })
    const [direction, setDirection] = createSignal<'down' | 'up' | 'right' | 'left'>('down')
    let triggerRef: HTMLDivElement | undefined

    const toggle = (e: MouseEvent) => {
        e.preventDefault()
        e.stopPropagation()
        console.log('Dropdown toggle', isOpen())

        if (!isOpen() && triggerRef) {
            const rect = triggerRef.getBoundingClientRect()
            const windowHeight = window.innerHeight

            // Simple logic: if below middle, open up. Else open down.
            // Can be improved with actual menu height measurement.
            if (rect.bottom > windowHeight * 0.6) {
                setDirection('up')
                setPosition({ x: rect.left, y: rect.top })
            } else {
                setDirection('down')
                setPosition({ x: rect.left, y: rect.bottom })
            }
        }
        setIsOpen(!isOpen())
    }

    const close = () => setIsOpen(false)

    // Click outside to close
    const handleClickOutside = () => {
        if (isOpen()) {
            close()
        }
    }

    onMount(() => {
        document.addEventListener('click', handleClickOutside)
    })

    onCleanup(() => {
        document.removeEventListener('click', handleClickOutside)
    })

    return (
        <>
            <div ref={triggerRef} class={`inline-block ${props.class || ''}`}>
                {props.trigger({ onClick: toggle, isOpen: isOpen() })}
            </div>

            <Show when={isOpen()}>
                <Portal>
                    <div
                        class='fixed z-50 min-w-[200px] bg-surface-container-high rounded-lg shadow-xl border border-outline-variant py-1 overflow-hidden'
                        style={{
                            left: `${position().x}px`,
                            top: direction() === 'down' ? `${position().y + 4}px` : 'auto',
                            bottom:
                                direction() === 'up'
                                    ? `${window.innerHeight - position().y + 4}px`
                                    : 'auto'
                        }}
                        onClick={e => e.stopPropagation()}
                    >
                        <For each={props.items}>
                            {item => (
                                <button
                                    class='w-full text-left px-4 py-2 text-sm text-on-surface hover:bg-surface-container-highest transition-colors flex items-center gap-2'
                                    onClick={() => {
                                        item.onClick?.()
                                        close()
                                    }}
                                >
                                    <Show when={item.icon}>
                                        <span class='text-on-surface-variant'>{item.icon}</span>
                                    </Show>
                                    <span>{item.label}</span>
                                </button>
                            )}
                        </For>
                    </div>
                </Portal>
            </Show>
        </>
    )
}
