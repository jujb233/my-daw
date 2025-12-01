import { Component, JSX, splitProps } from "solid-js";

interface SliderProps extends JSX.InputHTMLAttributes<HTMLInputElement> {
    label?: string;
    valueDisplay?: string | number;
}

export const Slider: Component<SliderProps> = (props) => {
    const [local, others] = splitProps(props, ["label", "valueDisplay", "class"]);

    return (
        <div class={`flex flex-col gap-1 ${local.class || ""}`}>
            <div class="flex justify-between items-center px-1">
                {local.label && (
                    <label class="text-xs text-on-surface-variant">{local.label}</label>
                )}
                {local.valueDisplay !== undefined && (
                    <span class="text-xs text-on-surface-variant">{local.valueDisplay}</span>
                )}
            </div>
            <input
                type="range"
                class="w-full h-1 bg-surface-container-highest rounded-lg appearance-none cursor-pointer accent-primary hover:accent-primary-container"
                {...others}
            />
        </div>
    );
};
