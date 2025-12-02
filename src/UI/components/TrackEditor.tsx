import { Component, For } from "solid-js";
import { store, addTrack, addClip, selectTrack } from "../../store";
import { t } from "../../i18n";
import { ClipInstance } from "../../store/types";
import { Button } from "../lib/Button";

const PIXELS_PER_BAR = 40;

const Ruler: Component = () => {
    return (
        <div class="h-8 bg-surface-container-high border-b border-outline-variant flex items-end sticky top-0 z-10">
            <div class="w-[200px] shrink-0 border-r border-outline-variant bg-surface-container-high flex items-center justify-center text-xs text-on-surface-variant">
                {t('tracks.header')}
            </div>
            <div class="flex-1 relative overflow-hidden min-w-[1000px]">
                <div class="absolute bottom-0 left-0 w-full h-full flex text-xs text-on-surface-variant font-mono">
                    <For each={Array.from({ length: 50 })}>
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

    return (
        <div
            class="absolute top-1 bottom-1 rounded-md flex items-center justify-center text-xs font-medium shadow-sm cursor-pointer hover:brightness-110 border overflow-hidden z-10"
            style={{
                left: `${(props.clip.startBar - 1) * PIXELS_PER_BAR}px`,
                width: `${props.clip.lengthBars * PIXELS_PER_BAR}px`,
                "background-color": `${content()?.color}80` || "#aec6ff80",
                "border-color": content()?.color || "#aec6ff",
                "color": "#e3e2e6"
            }}
            title={`${t('clip.title')}: ${content()?.name}`}
        >
            {content()?.name}
        </div>
    );
};

const TrackLane: Component<{ trackId: string }> = (props) => {
    const trackClips = () => store.clips.filter(c => c.trackId === props.trackId);

    const handleDoubleClick = (e: MouseEvent) => {
        const barIndex = Math.floor(e.offsetX / PIXELS_PER_BAR) + 1;
        // Snap to 4-bar grid
        const gridStart = Math.floor((barIndex - 1) / 4) * 4 + 1;
        addClip(props.trackId, gridStart, "Pattern A", "#aec6ff");
    };

    return (
        <div
            class="flex-1 h-24 border-b border-outline-variant bg-surface-container-lowest relative min-w-[1000px]"
            onDblClick={handleDoubleClick}
        >
            {/* Grid Lines (every 4 bars) */}
            <div class="absolute inset-0 pointer-events-none">
                <For each={Array.from({ length: 50 })}>
                    {(_, i) => (
                        <div
                            class="absolute top-0 bottom-0 border-r border-outline-variant/20"
                            style={{ left: `${(i() + 1) * 4 * PIXELS_PER_BAR}px` }}
                        ></div>
                    )}
                </For>
                {/* Bar lines (fainter) */}
                <For each={Array.from({ length: 200 })}>
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
    return (
        <div class="flex-1 flex flex-col overflow-hidden bg-surface-container-lowest relative">
            <Ruler />
            <div class="flex-1 overflow-auto">
                <div class="flex flex-col min-w-fit pb-20">
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
