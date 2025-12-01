import { Component } from "solid-js";
import { Surface } from "../lib/Surface";
import { Button } from "../lib/Button";
import { IconButton } from "../lib/IconButton";
import { Slider } from "../lib/Slider";
import { masterVolume, updateMasterVolume, waveform, updateWaveform } from "../../store/audio";

export const TimbreSidebar: Component<{ isOpen: boolean; onClose: () => void }> = (props) => {
    const handleVolumeChange = (e: Event) => {
        const val = parseFloat((e.target as HTMLInputElement).value);
        updateMasterVolume(val);
    };

    return (
        <div
            class={`transition-all duration-300 ease-in-out overflow-hidden flex flex-col border-l border-outline-variant bg-surface-container-low ${props.isOpen ? "w-80 opacity-100" : "w-0 opacity-0"
                }`}
        >
            <div class="h-14 flex items-center justify-between px-4 border-b border-outline-variant shrink-0">
                <span class="font-medium text-on-surface">Timbres</span>
                <IconButton onClick={props.onClose} variant="standard">
                    <svg xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 -960 960 960" width="24" fill="currentColor"><path d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z" /></svg>
                </IconButton>
            </div>

            <div class="flex-1 overflow-y-auto p-4 flex flex-col gap-3">
                <Surface level={1} class="p-4 flex flex-col gap-3">
                    <span class="text-sm font-medium text-on-surface">Master Volume</span>
                    <Slider
                        label="Gain"
                        min="0"
                        max="1"
                        step="0.01"
                        value={masterVolume()}
                        onInput={handleVolumeChange}
                        valueDisplay={`${Math.round(masterVolume() * 100)}%`}
                    />

                    <div class="flex flex-col gap-2 pt-2 border-t border-outline-variant">
                        <span class="text-sm font-medium text-on-surface">Waveform</span>
                        <div class="grid grid-cols-4 gap-1">
                            {[
                                { label: "Sine", value: 0 },
                                { label: "Sqr", value: 1 },
                                { label: "Saw", value: 2 },
                                { label: "Tri", value: 3 },
                            ].map((w) => (
                                <button
                                    class={`px-1 py-1.5 text-xs font-medium rounded transition-colors ${waveform() === w.value
                                            ? "bg-primary text-on-primary"
                                            : "bg-surface-container-highest text-on-surface-variant hover:bg-surface-container-high"
                                        }`}
                                    onClick={() => updateWaveform(w.value)}
                                >
                                    {w.label}
                                </button>
                            ))}
                        </div>
                    </div>
                </Surface>

                <Surface level={1} class="p-3 flex items-center gap-3 cursor-pointer hover:bg-surface-container-high transition-colors">
                    <div class="w-10 h-10 rounded bg-primary/20 flex items-center justify-center text-primary">
                        🎹
                    </div>
                    <div class="flex flex-col">
                        <span class="text-sm font-medium text-on-surface">Grand Piano</span>
                        <span class="text-xs text-on-surface-variant">Built-in</span>
                    </div>
                </Surface>

                <Surface level={1} class="p-3 flex items-center gap-3 cursor-pointer hover:bg-surface-container-high transition-colors">
                    <div class="w-10 h-10 rounded bg-tertiary/20 flex items-center justify-center text-tertiary">
                        🎻
                    </div>
                    <div class="flex flex-col">
                        <span class="text-sm font-medium text-on-surface">Violin Section</span>
                        <span class="text-xs text-on-surface-variant">Kontakt</span>
                    </div>
                </Surface>

                <Button variant="outlined" class="mt-2">Add Timbre</Button>
            </div>
        </div>
    );
};
