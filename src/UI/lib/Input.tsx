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
                <label class='text-xs text-on-surface-variant ml-3'>{local.label}</label>
            )}
            <input
                class='h-14 px-4 rounded-t-md bg-surface-container-highest border-b border-on-surface-variant text-on-surface focus:border-primary focus:border-b-2 outline-none transition-colors placeholder:text-on-surface-variant/50'
                {...others}
            />
            {local.error && <span class='text-xs text-error ml-3'>{local.error}</span>}
        </div>
    )
}
