import { Component, JSX, splitProps, createSignal } from "solid-js";

interface SliderProps extends Omit<JSX.InputHTMLAttributes<HTMLInputElement>, 'onChange'> {
    label?: string;
    valueDisplay?: string | number;
    value?: number;
    onChange?: (value: number) => void;
    min?: number;
    max?: number;
    step?: number;
}

export const Slider: Component<SliderProps> = (props) => {
    const [local] = splitProps(props, ["label", "valueDisplay", "class", "value", "onChange", "min", "max", "step"]);
    const [isDragging, setIsDragging] = createSignal(false);

    const min = local.min ?? 0;
    const max = local.max ?? 100;
    const step = local.step ?? 1;
    const value = local.value ?? min;

    const percentage = () => ((value - min) / (max - min)) * 100;

    const handlePointerDown = (e: PointerEvent) => {
        e.preventDefault();
        const target = e.currentTarget as HTMLElement;
        target.setPointerCapture(e.pointerId);
        setIsDragging(true);

        const rect = target.getBoundingClientRect();

        const updateValue = (clientX: number) => {
            const x = Math.max(0, Math.min(rect.width, clientX - rect.left));
            let pct = x / rect.width;

            let rawValue = min + pct * (max - min);

            // Snap to step
            if (step > 0) {
                rawValue = Math.round(rawValue / step) * step;
            }

            // Clamp
            let newValue = Math.max(min, Math.min(max, rawValue));

            if (local.onChange) {
                local.onChange(newValue);
            }
        };

        // Initial update on click
        updateValue(e.clientX);

        const onPointerMove = (moveEvent: PointerEvent) => {
            updateValue(moveEvent.clientX);
        };

        const onPointerUp = (upEvent: PointerEvent) => {
            target.releasePointerCapture(upEvent.pointerId);
            setIsDragging(false);
            target.removeEventListener("pointermove", onPointerMove);
            target.removeEventListener("pointerup", onPointerUp);
        };

        target.addEventListener("pointermove", onPointerMove);
        target.addEventListener("pointerup", onPointerUp);
    };

    return (
        <div class={`flex flex-col gap-2 touch-none select-none ${local.class || ""}`}>
            <div class="flex justify-between items-center px-1">
                {local.label && (
                    <label class="text-xs font-medium text-on-surface-variant">{local.label}</label>
                )}
                {local.valueDisplay !== undefined && (
                    <span class="text-xs font-mono text-on-surface-variant bg-surface-container px-1 rounded">
                        {local.valueDisplay}
                    </span>
                )}
            </div>

            <div
                class="h-8 relative flex items-center cursor-ew-resize"
                onPointerDown={handlePointerDown}
            >
                {/* Track Background */}
                <div class="w-full h-2 bg-surface-container-highest rounded-full overflow-hidden">
                    {/* Fill */}
                    <div
                        class="h-full bg-primary transition-all duration-75"
                        style={{ width: `${percentage()}%` }}
                    />
                </div>

                {/* Thumb (Larger for touch) */}
                <div
                    class={`absolute w-6 h-6 rounded-full shadow-sm border flex items-center justify-center transition-transform duration-100
                        ${isDragging()
                            ? "bg-primary-container border-primary scale-110"
                            : "bg-surface border-outline hover:border-primary"
                        }
                    `}
                    style={{
                        left: `${percentage()}%`,
                        transform: 'translateX(-50%)'
                    }}
                >
                    <div class={`w-2 h-2 rounded-full ${isDragging() ? "bg-primary" : "bg-on-surface-variant"}`} />
                </div>
            </div>
        </div>
    );
};
