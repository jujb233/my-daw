import { Component, JSX, splitProps } from 'solid-js'

interface InputProps extends JSX.InputHTMLAttributes<HTMLInputElement> {
    label?: string
    error?: string
}

export const Input: Component<InputProps> = props => {
    const [local, others] = splitProps(props, ['label', 'error', 'class'])

    return (
        <div class={`flex flex-col gap-1 ${local.class || ''}`}>
            {local.label && (
                <label class='text-on-surface-variant ml-3 text-xs'>{local.label}</label>
            )}
            <input
                class='bg-surface-container-highest border-on-surface-variant text-on-surface focus:border-primary placeholder:text-on-surface-variant/50 h-14 rounded-t-md border-b px-4 transition-colors outline-none focus:border-b-2'
                {...others}
            />
            {local.error && <span class='text-error ml-3 text-xs'>{local.error}</span>}
        </div>
    )
}
