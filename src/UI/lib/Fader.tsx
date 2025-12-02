import { Component, createSignal } from "solid-js";

interface FaderProps {
    value: number; // 0.0 to 1.0
    onChange: (val: number) => void;
    className?: string;
    defaultValue?: number;
}

export const Fader: Component<FaderProps> = (props) => {
    const [isDragging, setIsDragging] = createSignal(false);
    let trackRef: HTMLDivElement | undefined;

    const handlePointerDown = (e: PointerEvent) => {
        e.preventDefault();
        e.stopPropagation();
        const target = e.currentTarget as HTMLElement;
        target.setPointerCapture(e.pointerId);
        setIsDragging(true);

        const startY = e.clientY;
        const startValue = props.value;
        const sensitivity = 0.005; // Value change per pixel

        const onPointerMove = (moveEvent: PointerEvent) => {
            const deltaY = startY - moveEvent.clientY; // Up is positive
            const isFine = moveEvent.shiftKey;
            const currentSensitivity = isFine ? sensitivity * 0.1 : sensitivity;

            let newValue = startValue + deltaY * currentSensitivity;
            newValue = Math.max(0, Math.min(1, newValue));

            props.onChange(newValue);
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

    const handleDblClick = () => {
        props.onChange(props.defaultValue ?? 0.75);
    };

    return (
        <div
            class={`relative flex justify-center touch-none select-none ${props.className || ""}`}
            onDblClick={handleDblClick}
        >
            {/* Track Background */}
            <div
                ref={trackRef}
                class="h-full w-2 bg-surface-container-highest rounded-full relative overflow-hidden"
            >
                {/* Fill Level (Optional, maybe for meters, but faders usually just have a handle) */}
                <div
                    class="absolute bottom-0 left-0 right-0 bg-primary/20 pointer-events-none"
                    style={{ height: `${props.value * 100}%` }}
                />
            </div>

            {/* Hit Area & Handle Container */}
            <div
                class="absolute inset-0 cursor-ns-resize flex items-end justify-center"
                onPointerDown={handlePointerDown}
            >
                {/* Thumb / Handle */}
                <div
                    class={`w-8 h-12 rounded shadow-md border flex items-center justify-center transition-colors duration-75 absolute mb-[-24px]
                        ${isDragging()
                            ? "bg-primary-container border-primary"
                            : "bg-surface-container-high border-outline hover:border-outline-variant"
                        }
                    `}
                    style={{
                        bottom: `${props.value * 100}%`,
                        transform: 'translateY(50%)'
                    }}
                >
                    {/* Handle Grip Lines */}
                    <div class="flex flex-col gap-0.5 opacity-50">
                        <div class="w-4 h-[1px] bg-on-surface" />
                        <div class="w-4 h-[1px] bg-on-surface" />
                        <div class="w-4 h-[1px] bg-on-surface" />
                    </div>
                </div>
            </div>
        </div>
    );
};
