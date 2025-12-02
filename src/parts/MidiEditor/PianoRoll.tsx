import { Component, createEffect, createSignal, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { DawService } from "../../services/daw";
import { Ruler } from "./Ruler";
import { store, setStore, updateClipNotes } from "../../store";
import { IconButton } from "../../UI/lib/IconButton";
import { Note } from "../../store/types";

interface Clip {
    id: number;
    name: string;
    start_time: number;
    duration: number;
    instrument_ids: number[];
    instrument_routes: Record<number, number[]>;
    notes: Note[];
}

interface PianoRollProps {
    clipId: number;
}

export const PianoRoll: Component<PianoRollProps> = (props) => {
    const [clip, setClip] = createSignal<Clip | null>(null);
    const [zoom, setZoom] = createSignal(100); // pixels per beat (quarter note)
    let keysContainer: HTMLDivElement | undefined;
    let gridContainer: HTMLDivElement | undefined;

    const fetchClip = async () => {
        try {
            // Fetch from backend to get duration etc, but notes should come from store for consistency?
            // Actually, store.clipLibrary has the authoritative notes for "shared content".
            // But let's fetch the backend clip to get the instance-specific data (start time, duration).
            const c = await DawService.getClip(props.clipId);

            // Merge notes from store if available (Flyweight pattern)
            const instance = store.clips.find(cl => cl.id === props.clipId);
            if (instance) {
                const content = store.clipLibrary[instance.clipContentId];
                if (content) {
                    // Use content notes instead of backend notes (they should be synced, but store is source of truth for UI)
                    // Actually, let's trust the store for editing.
                    setClip({ ...c, notes: content.notes });
                    return;
                }
            }

            setClip(c);
        } catch (e) {
            console.error("Failed to fetch clip:", e);
        }
    };

    createEffect(() => {
        if (props.clipId !== undefined) {
            fetchClip();
        }
    });

    const addNote = async (time: number, pitch: number) => {
        if (!clip()) return;

        const newNote: Note = {
            relative_start: time,
            duration: 0.5, // Default 0.5s
            note: pitch,
            velocity: 0.8
        };

        const updatedNotes = [...clip()!.notes, newNote];

        // Optimistic update
        setClip({ ...clip()!, notes: updatedNotes });

        try {
            const instance = store.clips.find(c => c.id === props.clipId);
            if (instance) {
                await updateClipNotes(instance.clipContentId, updatedNotes);
            }
        } catch (e) {
            console.error("Failed to update clip notes:", e);
            fetchClip(); // Revert on error
        }
    };

    const removeNote = async (index: number) => {
        if (!clip()) return;

        const updatedNotes = clip()!.notes.filter((_, i) => i !== index);

        // Optimistic update
        setClip({ ...clip()!, notes: updatedNotes });

        try {
            const instance = store.clips.find(c => c.id === props.clipId);
            if (instance) {
                await updateClipNotes(instance.clipContentId, updatedNotes);
            }
        } catch (e) {
            console.error("Failed to update clip notes:", e);
            fetchClip();
        }
    };

    // Sync scroll
    const handleScroll = () => {
        if (keysContainer && gridContainer) {
            keysContainer.scrollTop = gridContainer.scrollTop;
        }
    };

    // Grid rendering helpers
    const NOTE_HEIGHT = 20;
    const KEYS = Array.from({ length: 128 }, (_, i) => 127 - i); // Top to bottom

    return (
        <div class="flex h-full w-full bg-surface-container-low overflow-hidden relative">
            {/* Zoom Controls */}
            <div class="absolute bottom-4 right-4 z-30 flex gap-2 bg-surface-container-high p-1 rounded-full shadow-md border border-outline-variant">
                <IconButton onClick={() => setZoom(z => Math.max(10, z * 0.8))} variant="standard" class="w-8 h-8">
                    <svg xmlns="http://www.w3.org/2000/svg" height="20" viewBox="0 -960 960 960" width="20" fill="currentColor"><path d="M200-440v-80h560v80H200Z" /></svg>
                </IconButton>
                <IconButton onClick={() => setZoom(z => Math.min(500, z * 1.25))} variant="standard" class="w-8 h-8">
                    <svg xmlns="http://www.w3.org/2000/svg" height="20" viewBox="0 -960 960 960" width="20" fill="currentColor"><path d="M440-440H200v-80h240v-240h80v240h240v80H520v240h-80v-240Z" /></svg>
                </IconButton>
            </div>

            {/* Piano Keys (Left) */}
            <div
                ref={keysContainer}
                class="w-16 flex-shrink-0 overflow-hidden border-r border-outline-variant bg-surface mt-8" // mt-8 to offset ruler
            >
                <div class="relative" style={{ height: `${KEYS.length * NOTE_HEIGHT}px` }}>
                    <For each={KEYS}>
                        {(note) => {
                            const isBlack = [1, 3, 6, 8, 10].includes(note % 12);
                            return (
                                <div
                                    class={`h-[20px] border-b border-outline-variant text-[10px] flex items-center justify-end pr-1 ${isBlack ? 'bg-surface-container-high text-on-surface-variant' : 'bg-surface text-on-surface'}`}
                                >
                                    {note % 12 === 0 ? `C${note / 12 - 1}` : ''}
                                </div>
                            );
                        }}
                    </For>
                </div>
            </div>

            {/* Grid Area */}
            <div
                ref={gridContainer}
                class="flex-1 overflow-auto relative bg-surface-container-lowest"
                onScroll={handleScroll}
            >
                {/* Ruler (Sticky) */}
                <div class="sticky top-0 z-20 bg-surface-container-high">
                    <Ruler
                        zoom={zoom()}
                        length={clip()?.duration || 10}
                        height={32}
                        onClick={(time) => {
                            if (clip()) {
                                const globalTime = clip()!.start_time + time;
                                invoke("seek", { position: globalTime });
                                // Update local store immediately for responsiveness
                                setStore("playback", "startTime", globalTime);
                            }
                        }}
                    />
                </div>

                <div
                    class="relative min-w-full"
                    style={{
                        height: `${KEYS.length * NOTE_HEIGHT}px`,
                        width: clip() ? `${clip()!.duration * zoom()}px` : '100%'
                    }}
                    onClick={(e) => {
                        // Adjust click coordinates for ruler height
                        const rect = e.currentTarget.getBoundingClientRect();
                        const x = e.clientX - rect.left;
                        const y = e.clientY - rect.top; // This is relative to the grid container content

                        const time = x / zoom();
                        const noteIndex = Math.floor(y / NOTE_HEIGHT);
                        const pitch = 127 - noteIndex;

                        addNote(time, pitch);
                    }}
                >
                    {/* Grid Lines */}
                    <div class="absolute inset-0 pointer-events-none opacity-10"
                        style={{
                            "background-image": `linear-gradient(to right, #888 1px, transparent 1px), linear-gradient(to bottom, #888 1px, transparent 1px)`,
                            "background-size": `${zoom()}px ${NOTE_HEIGHT}px`
                        }}
                    />

                    {/* Notes */}
                    <For each={clip()?.notes}>
                        {(note, i) => (
                            <div
                                class="absolute bg-primary rounded-sm border border-primary-container cursor-pointer hover:brightness-110"
                                style={{
                                    left: `${note.relative_start * zoom()}px`,
                                    top: `${(127 - note.note) * NOTE_HEIGHT}px`,
                                    width: `${note.duration * zoom()}px`,
                                    height: `${NOTE_HEIGHT - 1}px`
                                }}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    removeNote(i());
                                }}
                            />
                        )}
                    </For>                    {/* Playhead */}
                    <Show when={store.playback.startTime !== null && clip()}>
                        <div
                            class="absolute top-0 bottom-0 w-0.5 bg-tertiary z-10 pointer-events-none"
                            style={{
                                left: `${((store.playback.startTime || 0) - (clip()?.start_time || 0)) * zoom()}px`
                            }}
                        >
                            <div class="w-3 h-3 -ml-1.5 bg-tertiary transform rotate-45 -mt-1.5"></div>
                        </div>
                    </Show>
                </div>
            </div>
        </div>
    );
};
