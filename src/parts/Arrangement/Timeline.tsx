import { Component, For } from "solid-js";
import { GridClip } from "./GridClip";
import { IconButton } from "../../UI/lib/IconButton";
import { selectClip, store } from "../../store";

const PX_PER_BAR = 120; // Adjust as needed
const TOTAL_BARS = 100; // Fixed size for now

interface TrackRowProps {
    name: string;
    trackId: number;
    scrollLeft: number;
}

const TrackRow: Component<TrackRowProps> = (props) => {
    // Load clips from store
    const trackClips = () => store.clips.filter(c => c.trackId === props.trackId);

    const addClip = async () => {
        const name = "New Clip";
        const startBar = 1;

        // Calculate time in seconds for backend (assuming 120bpm 4/4 for now)
        // TODO: Use actual BPM/Sig conversion

        try {
            // We use the store action which handles backend + frontend state
            // But we need to import addClip from store/index.ts or use the one we defined?
            // The user's previous code used invoke directly in TrackRow. 
            // Let's switch to using the store's addClip if possible, or keep it simple.
            // The store has `addClip`. Let's use it.
            const { addClip } = await import("../../store");
            await addClip(props.trackId, startBar, name, "#10b981");
        } catch (e) {
            console.error("Failed to add clip:", e);
        }
    };

    return (
        <div class="flex h-24 border-b border-outline-variant bg-surface-container-low shrink-0">
            {/* Track Header */}
            <div class="w-48 border-r border-outline-variant p-2 flex flex-col justify-between bg-surface-container shrink-0 sticky left-0 z-10">
                <span class="font-medium text-on-surface">{props.name}</span>
                <div class="flex gap-1">
                    <IconButton variant="standard" class="w-6 h-6" onClick={addClip}>
                        <svg xmlns="http://www.w3.org/2000/svg" height="16" viewBox="0 -960 960 960" width="16" fill="currentColor"><path d="M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z" /></svg>
                    </IconButton>
                </div>
            </div>

            {/* Timeline Area */}
            <div class="relative bg-surface-container-lowest overflow-hidden" style={{ width: `${TOTAL_BARS * PX_PER_BAR}px` }}>
                {/* Grid Lines */}
                <div class="absolute inset-0 pointer-events-none opacity-10"
                    style={{
                        "background-image": `linear-gradient(to right, #888 1px, transparent 1px)`,
                        "background-size": `${PX_PER_BAR / 4}px 100%` // Quarter notes
                    }}
                />
                <div class="absolute inset-0 pointer-events-none opacity-20"
                    style={{
                        "background-image": `linear-gradient(to right, #888 1px, transparent 1px)`,
                        "background-size": `${PX_PER_BAR}px 100%` // Bars
                    }}
                />

                <For each={trackClips()}>
                    {(clip) => {
                        const clipData = () => store.clipLibrary[clip.clipContentId];
                        return (
                            <GridClip
                                name={clipData()?.name || "Clip"}
                                left={(clip.startBar - 1) * PX_PER_BAR}
                                width={clip.lengthBars * PX_PER_BAR}
                                color={clipData()?.color || "#666"}
                                onRemove={() => {
                                    import("../../store").then(m => m.deleteClip(clip.id));
                                }}
                                instrumentId={0} // TODO
                                targetTrackId={0} // TODO
                                onUpdate={(updates) => {
                                    if (updates.left !== undefined) {
                                        const newStartBar = Math.round(updates.left / PX_PER_BAR) + 1;
                                        import("../../store").then(m => m.updateClipPosition(clip.id, newStartBar));
                                    }
                                }}
                                onClick={() => selectClip(clip.id)}
                            />
                        );
                    }}
                </For>
            </div>
        </div>
    );
};

export const Timeline: Component = () => {
    let scrollContainer: HTMLDivElement | undefined;
    let rulerContainer: HTMLDivElement | undefined;

    const handleScroll = (e: Event) => {
        const target = e.target as HTMLDivElement;
        if (rulerContainer) rulerContainer.scrollLeft = target.scrollLeft;
        // If we had a separate header container for vertical scroll, we'd sync it here too
    };

    return (
        <div class="flex-1 flex flex-col overflow-hidden bg-surface">
            {/* Top Row: Header Placeholder + Ruler */}
            <div class="flex h-8 bg-surface-container-high border-b border-outline-variant shrink-0">
                {/* Corner */}
                <div class="w-48 border-r border-outline-variant shrink-0 bg-surface-container-high"></div>

                {/* Ruler */}
                <div
                    ref={rulerContainer}
                    class="flex-1 overflow-hidden whitespace-nowrap relative"
                >
                    <div class="h-full relative" style={{ width: `${TOTAL_BARS * PX_PER_BAR}px` }}>
                        <For each={Array(TOTAL_BARS).fill(0)}>
                            {(_, i) => (
                                <div
                                    class="absolute top-0 bottom-0 border-l border-on-surface-variant/50 text-[10px] pl-1 text-on-surface-variant flex items-end pb-1"
                                    style={{ left: `${i() * PX_PER_BAR}px` }}
                                >
                                    {i() + 1}
                                </div>
                            )}
                        </For>

                        {/* Playhead in Ruler */}
                        <div
                            class="absolute top-0 bottom-0 w-[1px] bg-primary z-20"
                            style={{ left: `${(store.playback.currentBar - 1) * PX_PER_BAR}px` }}
                        >
                            <div class="absolute -top-0 -left-[5px] w-0 h-0 border-l-[5px] border-l-transparent border-r-[5px] border-r-transparent border-t-[8px] border-t-primary"></div>
                        </div>
                    </div>
                </div>
            </div>

            {/* Main Content: Tracks */}
            <div
                ref={scrollContainer}
                onScroll={handleScroll}
                class="flex-1 overflow-auto relative"
            >
                <div class="min-w-fit">
                    <For each={store.tracks}>
                        {(track) => (
                            <TrackRow
                                name={track.name}
                                trackId={track.id}
                                scrollLeft={0}
                            />
                        )}
                    </For>

                    {/* Playhead Line Overlay */}
                    <div
                        class="absolute top-0 bottom-0 w-[2px] bg-primary pointer-events-none z-20"
                        style={{
                            left: `${(store.playback.currentBar - 1) * PX_PER_BAR + 192}px` // 192 is w-48
                        }}
                    />
                </div>
            </div>
        </div>
    );
};

