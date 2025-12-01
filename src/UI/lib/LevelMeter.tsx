import { Component } from "solid-js";

interface LevelMeterProps {
    level: number; // 0.0 to 1.0 (or higher for clipping)
    className?: string;
}

export const LevelMeter: Component<LevelMeterProps> = (props) => {
    // Convert linear level to dB for display? Or just linear for now.
    // Usually meters are logarithmic.
    // Let's assume input is linear amplitude.

    const heightPercent = () => Math.min(100, Math.max(0, props.level * 100));

    return (
        <div class={`w-4 bg-gray-900 rounded-sm overflow-hidden relative border border-gray-700 ${props.className || ""}`}>
            <div
                class="absolute bottom-0 left-0 right-0 bg-green-500 transition-all duration-75 ease-out"
                style={{ height: `${heightPercent()}%` }}
            />
            {/* Peak indicator or grid lines could go here */}
        </div>
    );
};
