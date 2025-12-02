import { Component, createSignal, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { Ruler } from "./Ruler";
import { store, setStore, updateClipNotes } from "../../store";
import { IconButton } from "../../UI/lib/IconButton";
import { Note } from "../../store/types";

interface PianoRollProps {
    clipId: number;
}

export const PianoRoll: Component<PianoRollProps> = (props) => {
    const [zoom, setZoom] = createSignal(100); // pixels per beat (quarter note)
    let keysContainer: HTMLDivElement | undefined;
    let gridContainer: HTMLDivElement | undefined;

    // Derived state from store
    const clipData = () => {
        const instance = store.clips.find(c => c.id === props.clipId);
        if (!instance) return null;
        const content = store.clipLibrary[instance.clipContentId];
        if (!content) return null;

        const bpm = store.info.bpm;
        const timeSig = store.info.timeSignature[0];

        return {
            instance,
            content,
            startBeats: (instance.startBar - 1) * timeSig,
            durationBeats: instance.lengthBars * timeSig,
            bpm
        };
    };

    const addNote = async (beat: number, pitch: number) => {
        const data = clipData();
        if (!data) return;

        // Constraint: Cannot add note outside clip duration
        if (beat < 0 || beat > data.durationBeats) return;

        const newNote: Note = {
            relative_start: beat,
            duration: Math.min(1.0, data.durationBeats - beat), // Default 1 beat or remainder
            note: pitch,
            velocity: 0.8
        };

        const updatedNotes = [...data.content.notes, newNote];

        // Update store (which updates UI immediately)
        setStore("clipLibrary", data.instance.clipContentId, "notes", updatedNotes);

        try {
            await updateClipNotes(data.instance.clipContentId, updatedNotes);
        } catch (e) {
            console.error("Failed to update clip notes:", e);
        }
    };

    const removeNote = async (index: number) => {
        const data = clipData();
        if (!data) return;

        const updatedNotes = data.content.notes.filter((_, i) => i !== index);
        setStore("clipLibrary", data.instance.clipContentId, "notes", updatedNotes);

        try {
            await updateClipNotes(data.instance.clipContentId, updatedNotes);
        } catch (e) {
            console.error("Failed to update clip notes:", e);
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
                        length={clipData()?.durationBeats || 10}
                        height={32}
                        onClick={(beat) => {
                            const data = clipData();
                            if (data) {
                                const globalBeats = data.startBeats + beat;
                                const globalTime = globalBeats * (60 / data.bpm);
                                invoke("seek", { position: globalTime });
                                setStore("playback", "startTime", globalTime);
                            }
                        }}
                    />
                </div>

                <div
                    class="relative min-w-full"
                    style={{
                        height: `${KEYS.length * NOTE_HEIGHT}px`,
                        width: clipData() ? `${clipData()!.durationBeats * zoom()}px` : '100%'
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
                    <For each={clipData()?.content.notes}>
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
                    </For>                    {/* Playhead - Only visible if within clip range */}
                    <Show when={(() => {
                        const data = clipData();
                        if (!data || store.playback.startTime === null) return false;
                        const currentBeats = store.playback.startTime * (data.bpm / 60);
                        const localBeats = currentBeats - data.startBeats;
                        return localBeats >= 0 && localBeats <= data.durationBeats;
                    })()}>
                        <div
                            class="absolute top-0 bottom-0 w-0.5 bg-tertiary z-10 pointer-events-none"
                            style={{
                                left: `${(() => {
                                    const data = clipData();
                                    if (!data) return 0;
                                    const currentBeats = (store.playback.startTime || 0) * (data.bpm / 60);
                                    return (currentBeats - data.startBeats) * zoom();
                                })()}px`
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
