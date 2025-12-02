import { Component, For } from "solid-js";

interface RulerProps {
    zoom: number; // pixels per unit
    length: number; // total units
    height?: number;
    start?: number; // start unit
    onClick?: (value: number) => void;
}

export const Ruler: Component<RulerProps> = (props) => {
    const markers = () => {
        const count = Math.ceil(props.length);
        return Array.from({ length: count + 1 }, (_, i) => (props.start || 0) + i);
    };

    return (
        <div
            class="bg-surface-container-high border-b border-outline-variant relative overflow-hidden select-none cursor-pointer"
            style={{ height: `${props.height || 32}px`, width: `${props.length * props.zoom}px` }}
            onClick={(e) => {
                const rect = e.currentTarget.getBoundingClientRect();
                const x = e.clientX - rect.left;
                const value = (x / props.zoom) + (props.start || 0);
                props.onClick?.(value);
            }}
        >
            <For each={markers()}>
                {(i) => (
                    <div
                        class="absolute bottom-0 border-l border-on-surface-variant/50 text-[10px] pl-1 text-on-surface-variant flex items-end pb-1"
                        style={{
                            left: `${(i - (props.start || 0)) * props.zoom}px`,
                            height: "50%"
                        }}
                    >
                        {i}
                    </div>
                )}
            </For>
        </div>
    );
};
