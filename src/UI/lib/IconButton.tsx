import { Component, JSX, splitProps } from "solid-js";

type IconButtonVariant = "filled" | "tonal" | "outlined" | "standard" | "ghost";

interface IconButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
    variant?: IconButtonVariant;
}

export const IconButton: Component<IconButtonProps> = (props) => {
    const [local, others] = splitProps(props, ["variant", "class", "children"]);

    const baseClass = "h-10 w-10 rounded-full flex items-center justify-center transition-colors duration-200 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed";

    const variants = {
        filled: "bg-primary text-on-primary hover:bg-primary/90 active:bg-primary/80",
        tonal: "bg-secondary-container text-on-secondary-container hover:bg-secondary-container/90 active:bg-secondary-container/80",
        outlined: "border border-outline text-on-surface-variant hover:bg-on-surface-variant/10 active:bg-on-surface-variant/20",
        standard: "text-on-surface-variant hover:bg-on-surface-variant/10 active:bg-on-surface-variant/20",
        ghost: "text-on-surface-variant hover:bg-on-surface-variant/10 active:bg-on-surface-variant/20" // Same as standard for now
    };

    return (
        <button
            class={`${baseClass} ${variants[local.variant || "standard"]} ${local.class || ""}`}
            {...others}
        >
            {local.children}
        </button>
    );
};
