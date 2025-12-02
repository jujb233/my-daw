import { Component, For, createSignal } from "solid-js";
import { GridClip } from "./GridClip";
import { IconButton } from "../../lib/IconButton";
import { instances } from "../../../store/audio";
import { invoke } from "@tauri-apps/api/core";
import { selectClip } from "../../../store";

interface TrackRowProps {
    name: string;
    height?: number;
}

const TrackRow: Component<TrackRowProps> = (props) => {
    const [clips, setClips] = createSignal([
        { id: 1, name: "贝斯线", start: 50, length: 200, color: "#8b5cf6" },
        { id: 2, name: "旋律 A", start: 300, length: 150, color: "#ec4899" }
    ]);

    const addClip = async () => {
        // Default values for new clip
        const name = "新片段";
        const start = 100;
        const length = 100;
        const instrumentId = 0; // Default to first instrument
        const targetTrackIds = [0]; // Default to first track

        try {
            // Call backend to add clip
            // Note: start_time and duration in backend are in seconds. 
            // Frontend pixels need conversion. Let's assume 100px = 1 second for now.
            const id = await invoke("add_clip", {
                name,
                startTime: start / 100.0,
                duration: length / 100.0,
                instrumentId,
                targetTrackIds
            }) as number;

            setClips(prev => [...prev, {
                id, // Use backend ID
                name,
                start,
                length,
                color: "#10b981"
            }]);
        } catch (e) {
            console.error("Failed to add clip:", e);
        }
    };

    const removeClip = (id: number) => {
        setClips(prev => prev.filter(c => c.id !== id));
    };

    const updateClip = async (id: number, updates: any) => {
        try {
            // Prepare backend updates
            const backendUpdates: any = { id };
            if (updates.instrumentId !== undefined) backendUpdates.instrumentId = updates.instrumentId;
            if (updates.targetTrackId !== undefined) backendUpdates.targetTrackIds = [updates.targetTrackId];

            await invoke("update_clip", backendUpdates);

            // Update local state
            setClips(prev => prev.map(c => {
                if (c.id === id) {
                    return { ...c, ...updates };
                }
                return c;
            }));
        } catch (e) {
            console.error("Failed to update clip:", e);
        }
    };

    return (
        <div class="flex h-24 border-b border-outline-variant bg-surface-container-low">
            {/* Track Header */}
            <div class="w-48 border-r border-outline-variant p-2 flex flex-col justify-between bg-surface-container">
                <span class="font-medium text-on-surface">{props.name}</span>

                <div class="flex gap-1">
                    <IconButton variant="standard" class="w-6 h-6" onClick={addClip}>
                        <svg xmlns="http://www.w3.org/2000/svg" height="16" viewBox="0 -960 960 960" width="16" fill="currentColor"><path d="M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z" /></svg>
                    </IconButton>
                </div>
            </div>

            {/* Timeline Area */}
            <div class="flex-1 relative bg-surface-container-lowest overflow-hidden">
                {/* Grid Lines (Visual only for now) */}
                <div class="absolute inset-0 pointer-events-none opacity-10"
                    style={{
                        "background-image": "linear-gradient(to right, #888 1px, transparent 1px)",
                        "background-size": "50px 100%"
                    }}
                />

                <For each={clips()}>
                    {(clip) => (
                        <GridClip
                            name={clip.name}
                            left={clip.start}
                            width={clip.length}
                            color={clip.color}
                            onRemove={() => removeClip(clip.id)}
                            // Pass default or stored values
                            instrumentId={(clip as any).instrumentId ?? 0}
                            targetTrackId={(clip as any).targetTrackId ?? 0}
                            onUpdate={(updates) => updateClip(clip.id, updates)}
                            onClick={() => selectClip(clip.id)}
                        />
                    )}
                </For>
            </div>
        </div>
    );
}; export const Timeline: Component = () => {
    return (
        <div class="flex-1 flex flex-col overflow-hidden bg-surface">
            {/* Ruler */}
            <div class="h-8 bg-surface-container-high border-b border-outline-variant flex items-end pl-48">
                {/* Simple ruler markers */}
                <div class="flex-1 h-1/2 flex justify-between px-2">
                    <For each={Array(20).fill(0)}>
                        {(_, i) => (
                            <div class="h-full border-l border-on-surface-variant/50 text-[10px] pl-1 text-on-surface-variant">
                                {i() + 1}
                            </div>
                        )}
                    </For>
                </div>
            </div>

            {/* Tracks Area */}
            <div class="flex-1 overflow-y-auto">
                <TrackRow name="轨道 1" />
                <TrackRow name="轨道 2" />
                <TrackRow name="轨道 3" />
                <TrackRow name="轨道 4" />
            </div>
        </div>
    );
};
