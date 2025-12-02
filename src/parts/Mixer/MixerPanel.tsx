import { Component, For, createEffect, onCleanup, createSignal } from "solid-js";
import { mixerTracks, fetchMixerTracks, addMixerTrack, startMetering, stopMetering, meterLevels } from "../../store/mixer";
import { MixerStrip } from "./MixerStrip";
import { IconButton } from "../../UI/lib/IconButton";
import { t } from "../../i18n";

export const MixerPanel: Component = () => {
    const [isOpen, setIsOpen] = createSignal(false);
    const [width, setWidth] = createSignal(600);
    const [isResizing, setIsResizing] = createSignal(false);

    createEffect(() => {
        fetchMixerTracks();
        startMetering();
        onCleanup(() => stopMetering());
    });

    const startResize = (e: MouseEvent | TouchEvent) => {
        e.preventDefault();
        setIsResizing(true);
        const startX = 'touches' in e ? e.touches[0].clientX : e.clientX;
        const startWidth = width();

        const handleMove = (moveEvent: MouseEvent | TouchEvent) => {
            const currentX = 'touches' in moveEvent ? moveEvent.touches[0].clientX : moveEvent.clientX;
            const newWidth = Math.max(200, Math.min(window.innerWidth - 100, startWidth + (currentX - startX)));
            setWidth(newWidth);
        };

        const handleUp = () => {
            setIsResizing(false);
            window.removeEventListener('mousemove', handleMove);
            window.removeEventListener('mouseup', handleUp);
            window.removeEventListener('touchmove', handleMove);
            window.removeEventListener('touchend', handleUp);
        };

        window.addEventListener('mousemove', handleMove);
        window.addEventListener('mouseup', handleUp);
        window.addEventListener('touchmove', handleMove);
        window.addEventListener('touchend', handleUp);
    };

    return (
        <div
            class={`relative z-20 bg-surface border-r border-outline-variant transition-all duration-300 ease-in-out flex flex-col shadow-xl h-full shrink-0 ${isOpen() ? "" : "w-12"}`}
            style={{ width: isOpen() ? `${width()}px` : undefined }}
        >
            {/* Toggle Handle - Centered on Right Edge */}
            <div
                class="absolute -right-4 top-1/2 -translate-y-1/2 w-4 h-16 bg-surface border border-l-0 border-outline-variant rounded-r cursor-pointer flex items-center justify-center hover:bg-surface-container-high z-50"
                onClick={() => setIsOpen(!isOpen())}
                title={isOpen() ? "Collapse" : "Expand"}
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

            {/* Resize Handle */}
            <div
                class={`absolute right-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-primary z-40 ${isResizing() ? "bg-primary" : "bg-transparent"}`}
                onMouseDown={startResize}
                onTouchStart={startResize}
                style={{ display: isOpen() ? 'block' : 'none' }}
            ></div>

            {/* Collapsed View: Just a vertical label or icon */}
            <div class={`h-full w-12 flex flex-col items-center pt-4 gap-4 ${isOpen() ? "hidden" : "flex"}`}>
                <span class="writing-vertical-rl text-on-surface font-medium tracking-widest select-none" style={{ "text-orientation": "upright" }}>{t('mixer.label')}</span>
                <IconButton onClick={() => setIsOpen(true)} variant="standard">
                    <svg xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 -960 960 960" width="24" fill="currentColor"><path d="M120-320v-80h280v80H120Zm0-160v-80h440v80H120Zm0-160v-80h440v80H120Zm520 480v-160H480v-80h160v-160h80v160h160v80H720v160h-80Z" /></svg>
                </IconButton>
            </div>

            {/* Expanded View */}
            <div class={`flex-1 flex flex-col overflow-hidden ${isOpen() ? "flex" : "hidden"}`}>
                <div class="h-10 border-b border-outline-variant flex items-center justify-between px-4 bg-surface-container shrink-0">
                    <span class="font-bold text-on-surface select-none">{t('mixer.console')}</span>
                    <div class="flex gap-2">
                        {/* Maximize Button */}
                        <IconButton onClick={() => setWidth(window.innerWidth - 350)} variant="standard" class="w-8 h-8" title="Maximize">
                            <svg xmlns="http://www.w3.org/2000/svg" height="18" viewBox="0 -960 960 960" width="18" fill="currentColor"><path d="M120-120v-320h80v184l504-504H520v-80h320v320h-80v-184L256-120H120Z" /></svg>
                        </IconButton>
                        <IconButton onClick={addMixerTrack} variant="filled" class="w-8 h-8">
                            <svg xmlns="http://www.w3.org/2000/svg" height="20" viewBox="0 -960 960 960" width="20" fill="currentColor"><path d="M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z" /></svg>
                        </IconButton>
                    </div>
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
