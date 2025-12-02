import { Component, For, createEffect, onCleanup, createSignal } from "solid-js";
import { mixerTracks, fetchMixerTracks, addMixerTrack, startMetering, stopMetering, meterLevels } from "../../store/mixer";
import { MixerStrip } from "./MixerStrip";
import { IconButton } from "../../UI/lib/IconButton";
import { t } from "../../i18n";

export const MixerPanel: Component = () => {
    const [isOpen, setIsOpen] = createSignal(false);

    createEffect(() => {
        fetchMixerTracks();
        startMetering();
        onCleanup(() => stopMetering());
    });

    return (
        <div
            class={`fixed left-0 bottom-0 top-14 z-20 bg-surface border-r border-outline-variant transition-all duration-300 ease-in-out flex flex-col shadow-xl ${isOpen() ? "w-[600px]" : "w-12"
                }`}
        >
            {/* Toggle Handle */}
            <div
                class="absolute -right-4 top-1/2 -translate-y-1/2 w-4 h-16 bg-surface border border-l-0 border-outline-variant rounded-r cursor-pointer flex items-center justify-center hover:bg-surface-container-high"
                onClick={() => setIsOpen(!isOpen())}
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    height="24"
                    viewBox="0 -960 960 960"
                    width="24"
                    fill="currentColor"
                    class={`transition-transform duration-300 ${isOpen() ? "rotate-180" : ""}`}
                >
                    <path d="M504-480 320-664l56-56 240 240-240 240-56-56 184-184Z" />
                </svg>
            </div>

            {/* Collapsed View: Just a vertical label or icon */}
            <div class={`h-full w-12 flex flex-col items-center pt-4 gap-4 ${isOpen() ? "hidden" : "flex"}`}>
                <span class="writing-vertical-rl text-on-surface font-medium tracking-widest rotate-180">{t('mixer.label')}</span>
                <IconButton onClick={() => setIsOpen(true)} variant="standard">
                    <svg xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 -960 960 960" width="24" fill="currentColor"><path d="M120-320v-80h280v80H120Zm0-160v-80h440v80H120Zm0-160v-80h440v80H120Zm520 480v-160H480v-80h160v-160h80v160h160v80H720v160h-80Z" /></svg>
                </IconButton>
            </div>

            {/* Expanded View */}
            <div class={`flex-1 flex flex-col overflow-hidden ${isOpen() ? "flex" : "hidden"}`}>
                <div class="h-10 border-b border-outline-variant flex items-center justify-between px-4 bg-surface-container">
                    <span class="font-bold text-on-surface">{t('mixer.console')}</span>
                    <IconButton onClick={addMixerTrack} variant="filled" class="w-8 h-8">
                        <svg xmlns="http://www.w3.org/2000/svg" height="20" viewBox="0 -960 960 960" width="20" fill="currentColor"><path d="M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z" /></svg>
                    </IconButton>
                </div>

                <div class="flex-1 overflow-x-auto flex">
                    <For each={mixerTracks()}>
                        {(track) => (
                            <MixerStrip
                                track={track}
                                level={track.meter_id ? (meterLevels()[track.meter_id] || 0) : 0}
                            />
                        )}
                    </For>
                </div>
            </div>
        </div>
    );
};
