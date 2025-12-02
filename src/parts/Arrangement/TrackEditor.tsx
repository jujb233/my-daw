import { Component, For, createSignal } from "solid-js";
import { store, setStore, addTrack, addClip, selectTrack, selectClip } from "../../store";
import { t } from "../../i18n";
import { ClipInstance } from "../../store/types";
import { Button } from "../../UI/lib/Button";
import { invoke } from "@tauri-apps/api/core";

const PIXELS_PER_BAR = 60;

const Playhead: Component = () => {
    const left = () => (store.playback.currentBar - 1) * PIXELS_PER_BAR;

    return (
        <div
            class="absolute top-0 bottom-0 w-[1px] bg-red-500 z-50 pointer-events-none"
            style={{ left: `${left()}px` }}
        >
            <div class="absolute -top-1 -left-1.5 w-3 h-3 bg-red-500 rotate-45"></div>
        </div>
    );
};

const Ruler: Component<{ scrollRef: (el: HTMLDivElement) => void }> = (props) => {
    return (
        <div class="h-8 bg-surface-container-high border-b border-outline-variant flex items-end sticky top-0 z-10 shrink-0">
            <div class="w-[200px] shrink-0 border-r border-outline-variant bg-surface-container-high flex items-center justify-center text-xs text-on-surface-variant">
                {t('tracks.header')}
            </div>
            <div
                ref={props.scrollRef}
                class="flex-1 relative overflow-hidden whitespace-nowrap"
            >
                {/* Container for ruler content that matches track width */}
                <div class="h-full relative min-w-[4000px]">
                    <div class="absolute bottom-0 left-0 w-full h-full flex text-xs text-on-surface-variant font-mono">
                        <For each={Array.from({ length: 100 })}>
                            {(_, i) => {
                                const barNum = i() + 1;
                                return (
                                    <div
                                        class="absolute bottom-0 border-l border-outline-variant/50 pl-1 h-4 flex items-center"
                                        style={{ left: `${(barNum - 1) * PIXELS_PER_BAR}px` }}
                                    >
                                        {barNum}
                                    </div>
                                );
                            }}
                        </For>
                    </div>
                    {/* Playhead Marker in Ruler */}
                    <div
                        class="absolute bottom-0 h-4 w-[1px] bg-red-500 z-50 pointer-events-none"
                        style={{ left: `${(store.playback.currentBar - 1) * PIXELS_PER_BAR}px` }}
                    >
                        <div class="absolute -top-1 -left-1.5 w-3 h-3 bg-red-500 rotate-45"></div>
                    </div>
                </div>
            </div>
        </div>
    );
};

const TrackHeader: Component<{ track: any }> = (props) => {
    return (
        <div
            class={`w-[200px] shrink-0 h-24 border-r border-b border-outline-variant flex flex-col p-2 gap-2 relative group cursor-pointer transition-colors ${store.selectedTrackId === props.track.id ? 'bg-secondary-container/30' : 'bg-surface-container-low'}`}
            onClick={() => selectTrack(props.track.id)}
        >
            <div class="absolute left-0 top-0 bottom-0 w-1" style={{ "background-color": props.track.color }}></div>
            <div class="flex justify-between items-center pl-2">
                <span class="font-medium text-sm truncate text-on-surface">{props.track.name}</span>
                <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <div title={t('icons.mute')} class={`w-4 h-4 rounded text-[10px] flex items-center justify-center cursor-pointer ${props.track.muted ? 'bg-primary text-on-primary' : 'bg-on-surface-variant/20 hover:bg-primary hover:text-on-primary'}`}>M</div>
                    <div title={t('icons.solo')} class={`w-4 h-4 rounded text-[10px] flex items-center justify-center cursor-pointer ${props.track.soloed ? 'bg-tertiary text-on-tertiary' : 'bg-on-surface-variant/20 hover:bg-tertiary hover:text-on-tertiary'}`}>S</div>
                </div>
            </div>
            <div class="flex-1 bg-surface-container-highest rounded opacity-50">
                {/* Mini visualizer placeholder */}
            </div>
        </div>
    );
};

const ClipView: Component<{ clip: ClipInstance }> = (props) => {
    const content = () => store.clipLibrary[props.clip.clipContentId];
    const [isDragging, setIsDragging] = createSignal(false);
    const [dragOffset, setDragOffset] = createSignal(0);

    const handleMouseDown = (e: MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
        selectClip(props.clip.id);
        setIsDragging(true);
        const startX = e.clientX;

        const handleMouseMove = (moveEvent: MouseEvent) => {
            setDragOffset(moveEvent.clientX - startX);
        };

        const handleMouseUp = async () => {
            setIsDragging(false);
            window.removeEventListener('mousemove', handleMouseMove);
            window.removeEventListener('mouseup', handleMouseUp);

            const offsetBars = dragOffset() / PIXELS_PER_BAR;
            // Snap to grid (optional, but good for DAW) - let's snap to 0.25 bar (1 beat)
            const rawNewStartBar = props.clip.startBar + offsetBars;
            const newStartBar = Math.max(1, Math.round(rawNewStartBar * 4) / 4);

            // Calculate time
            const bpm = store.info.bpm;
            const timeSig = store.info.timeSignature[0];
            const newStartTime = (newStartBar - 1) * timeSig * (60 / bpm);

            try {
                await invoke("update_clip", {
                    id: props.clip.id,
                    startTime: newStartTime
                });

                // Update local store
                setStore("clips", (clips) =>
                    clips.map((c) =>
                        c.id === props.clip.id ? { ...c, startBar: newStartBar } : c
                    )
                );
            } catch (err) {
                console.error("Failed to update clip:", err);
            }

            setDragOffset(0);
        };

        window.addEventListener('mousemove', handleMouseMove);
        window.addEventListener('mouseup', handleMouseUp);
    };

    return (
        <div
            class="absolute top-1 bottom-1 rounded-md flex flex-col justify-center px-2 text-xs font-medium shadow-sm cursor-pointer hover:brightness-110 border overflow-hidden"
            style={{
                left: `${(props.clip.startBar - 1) * PIXELS_PER_BAR + dragOffset()}px`,
                width: `${props.clip.lengthBars * PIXELS_PER_BAR}px`,
                "background-color": `${content()?.color}80` || "#aec6ff80",
                "border-color": content()?.color || "#aec6ff",
                "color": "#e3e2e6",
                "z-index": isDragging() ? 100 : 10,
                "cursor": isDragging() ? "grabbing" : "grab"
            }}
            title={`${t('clip.title')}: ${content()?.name}`}
            onMouseDown={handleMouseDown}
        >
            <span class="truncate">{content()?.name}</span>
            <span class="text-[10px] opacity-70 truncate">Bar {props.clip.startBar}</span>
        </div>
    );
};

const TrackLane: Component<{ trackId: number }> = (props) => {
    const trackClips = () => store.clips.filter(c => c.trackId === props.trackId);

    const handleDoubleClick = (e: MouseEvent) => {
        const barIndex = Math.floor(e.offsetX / PIXELS_PER_BAR) + 1;
        // Snap to 4-bar grid
        const gridStart = Math.floor((barIndex - 1) / 4) * 4 + 1;
        addClip(props.trackId, gridStart, "Pattern A", "#aec6ff");
    };

    return (
        <div
            class="flex-1 h-24 border-b border-outline-variant bg-surface-container-lowest relative min-w-[4000px]"
            onDblClick={handleDoubleClick}
        >
            {/* Grid Lines (every 4 bars) */}
            <div class="absolute inset-0 pointer-events-none">
                <For each={Array.from({ length: 100 })}>
                    {(_, i) => (
                        <div
                            class="absolute top-0 bottom-0 border-r border-outline-variant/20"
                            style={{ left: `${(i() + 1) * 4 * PIXELS_PER_BAR}px` }}
                        ></div>
                    )}
                </For>
                {/* Bar lines (fainter) */}
                <For each={Array.from({ length: 400 })}>
                    {(_, i) => (
                        <div
                            class="absolute top-0 bottom-0 border-r border-outline-variant/5"
                            style={{ left: `${(i() + 1) * PIXELS_PER_BAR}px` }}
                        ></div>
                    )}
                </For>
            </div>

            {/* Clips */}
            <For each={trackClips()}>
                {(clip) => <ClipView clip={clip} />}
            </For>
        </div>
    );
};

export const TrackEditor: Component = () => {
    let scrollContainer: HTMLDivElement | undefined;
    let rulerContainer: HTMLDivElement | undefined;

    const handleScroll = (e: Event) => {
        const target = e.target as HTMLDivElement;
        if (rulerContainer) {
            rulerContainer.scrollLeft = target.scrollLeft;
        }
    };

    return (
        <div class="flex-1 flex flex-col overflow-hidden bg-surface-container-lowest relative">
            <Ruler scrollRef={(el) => rulerContainer = el} />
            <div
                ref={scrollContainer}
                onScroll={handleScroll}
                class="flex-1 overflow-auto"
            >
                <div class="flex flex-col min-w-fit pb-20 relative">
                    <Playhead />
                    <For each={store.tracks}>
                        {(track) => (
                            <div class="flex">
                                <TrackHeader track={track} />
                                <TrackLane trackId={track.id} />
                            </div>
                        )}
                    </For>

                    <div class="p-4">
                        <Button variant="tonal" onClick={addTrack}>{t('tracks.addTrack')}</Button>
                    </div>
                </div>
            </div>
        </div>
    );
};

