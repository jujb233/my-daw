import { Component, JSX, splitProps } from 'solid-js'

type ButtonVariant = 'filled' | 'tonal' | 'outlined' | 'text'

interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
    variant?: ButtonVariant
}

export const Button: Component<ButtonProps> = props => {
    const [local, others] = splitProps(props, ['variant', 'class', 'children'])

    const baseClass =
        'h-10 px-6 rounded-full text-sm font-medium transition-colors duration-200 flex items-center justify-center cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed'

    const variants = {
        filled: 'bg-primary text-on-primary hover:bg-primary/90 active:bg-primary/80',
        tonal: 'bg-secondary-container text-on-secondary-container hover:bg-secondary-container/90 active:bg-secondary-container/80',
        outlined: 'border border-outline text-primary hover:bg-primary/10 active:bg-primary/20',
        text: 'text-primary hover:bg-primary/10 active:bg-primary/20 px-3'
    }

    return (
        <button
            class={`${baseClass} ${variants[local.variant || 'filled']} ${local.class || ''}`}
            {...others}
        >
            {local.children}
        </button>
    )
}
