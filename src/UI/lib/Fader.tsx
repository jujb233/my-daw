import { Component } from "solid-js";

interface FaderProps {
    value: number; // 0.0 to 1.0
    onChange: (val: number) => void;
    className?: string;
}

export const Fader: Component<FaderProps> = (props) => {
    return (
        <div class={`h-32 w-8 bg-gray-800 rounded-full relative flex justify-center ${props.className || ""}`}>
            <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={props.value}
                onInput={(e) => props.onChange(parseFloat(e.currentTarget.value))}
                class="absolute w-32 h-8 -rotate-90 origin-center top-12 cursor-pointer opacity-0 z-10"
            />
            {/* Visual Track */}
            <div class="w-1 h-full bg-gray-900 rounded-full absolute top-0" />

            {/* Thumb */}
            <div
                class="w-6 h-4 bg-gray-400 rounded shadow-md absolute border border-gray-600 pointer-events-none"
                style={{ bottom: `calc(${props.value * 100}% - 8px)` }}
            />
        </div>
    );
};
