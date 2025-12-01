import { Component, createEffect, createSignal, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface Note {
    relative_start: number;
    duration: number;
    note: number;
    velocity: number;
}

interface Clip {
    id: number;
    name: string;
    start_time: number;
    duration: number;
    instrument_id: number;
    target_track_ids: number[];
    notes: Note[];
}

interface PianoRollProps {
    clipId: number;
}

export const PianoRoll: Component<PianoRollProps> = (props) => {
    const [clip, setClip] = createSignal<Clip | null>(null);
    const [zoom, setZoom] = createSignal(100); // pixels per second

    const fetchClip = async () => {
        try {
            const c = await invoke<Clip>("get_clip", { id: props.clipId });
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
            await invoke("update_clip", {
                id: clip()!.id,
                notes: updatedNotes
            });
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
            await invoke("update_clip", {
                id: clip()!.id,
                notes: updatedNotes
            });
        } catch (e) {
            console.error("Failed to update clip notes:", e);
            fetchClip();
        }
    };

    // Grid rendering helpers
    const NOTE_HEIGHT = 20;
    const KEYS = Array.from({ length: 128 }, (_, i) => 127 - i); // Top to bottom

    return (
        <div class="flex h-full w-full bg-surface-container-low overflow-hidden">
            {/* Piano Keys (Left) */}
            <div class="w-16 flex-shrink-0 overflow-y-hidden border-r border-outline-variant bg-surface">
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
            <div class="flex-1 overflow-auto relative bg-surface-container-lowest">
                <div
                    class="relative min-w-full"
                    style={{
                        height: `${KEYS.length * NOTE_HEIGHT}px`,
                        width: clip() ? `${clip()!.duration * zoom()}px` : '100%'
                    }}
                    onClick={(e) => {
                        const rect = e.currentTarget.getBoundingClientRect();
                        const x = e.clientX - rect.left;
                        const y = e.clientY - rect.top;

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
                    </For>
                </div>
            </div>
        </div>
    );
};
