import { Component, createSignal, For, Show } from 'solid-js'
import { store, updateClip } from '../../store'
import { IconButton } from '../../UI/lib/IconButton'
import { PianoRollNote } from './PianoRollNote'
import { defaultTimeService, PPQ, SnapGrid } from '../../services/time'
import { Note, MusicalLength } from '../../store/model'

interface PianoRollProps {
    clipId: string
}

export const PianoRoll: Component<PianoRollProps> = props => {
    const [zoom, setZoom] = createSignal(100) // pixels per beat
    let keysContainer: HTMLDivElement | undefined
    let gridContainer: HTMLDivElement | undefined

    const clip = () => store.clips.find(c => c.id === props.clipId)

    const pixelsPerTick = () => zoom() / PPQ
    const NOTE_HEIGHT = 20
    const KEYS = Array.from({ length: 128 }, (_, i) => 127 - i)

    // Cursor Logic
    const cursorPosition = () => {
        const c = clip()
        if (!c || !store.playback.isPlaying) return null

        const currentTime = store.playback.currentPosition.time
        const clipStart = defaultTimeService.ticksToSeconds(
            defaultTimeService.positionToTicks(c.start)
        )
        const clipEnd = clipStart + defaultTimeService.ticksToSeconds(c.length.totalTicks)

        if (currentTime >= clipStart && currentTime <= clipEnd) {
            const relativeTime = currentTime - clipStart
            const relativeTicks = defaultTimeService.secondsToTicks(relativeTime)
            return relativeTicks * pixelsPerTick()
        }
        return null
    }

    const handleNoteUpdate = (
        noteId: string,
        newStartPx: number,
        newWidthPx: number,
        newNoteVal?: number
    ) => {
        const c = clip()
        if (!c) return

        const ppt = pixelsPerTick()
        let startTicks = newStartPx / ppt
        let durationTicks = newWidthPx / ppt

        // Snap
        const snap = (store.snapInterval || '1/16') as SnapGrid
        startTicks = defaultTimeService.snapTicks(startTicks, snap)
        durationTicks = defaultTimeService.snapTicks(durationTicks, snap)

        startTicks = Math.max(0, startTicks)
        durationTicks = Math.max(PPQ / 16, durationTicks)

        const newStart = defaultTimeService.ticksToPosition(startTicks)
        const newDuration: MusicalLength = {
            bars: 0,
            beats: 0,
            sixteenths: 0,
            ticks: 0,
            totalTicks: durationTicks,
            seconds: defaultTimeService.ticksToSeconds(durationTicks)
        }

        const updatedNotes = c.notes.map(n => {
            if (n.id === noteId) {
                return {
                    ...n,
                    start: newStart,
                    duration: newDuration,
                    note: newNoteVal !== undefined ? newNoteVal : n.note
                }
            }
            return n
        })

        updateClip(c.id, { notes: updatedNotes })
    }

    const handleGridClick = (e: MouseEvent) => {
        if (e.button !== 0) return
        const c = clip()
        if (!c) return

        const rect = (e.currentTarget as HTMLDivElement).getBoundingClientRect()
        const x = e.clientX - rect.left
        const y = e.clientY - rect.top + (gridContainer?.scrollTop || 0)

        const ppt = pixelsPerTick()
        let tick = x / ppt
        const snap = (store.snapInterval || '1/16') as SnapGrid
        tick = defaultTimeService.snapTicks(tick, snap)

        const noteIndex = Math.floor(y / NOTE_HEIGHT)
        const pitch = 127 - noteIndex

        if (pitch < 0 || pitch > 127) return

        const newNote: Note = {
            id: Math.random().toString(36).substr(2, 9),
            note: pitch,
            start: defaultTimeService.ticksToPosition(tick),
            duration: {
                bars: 0,
                beats: 0,
                sixteenths: 0,
                ticks: 0,
                totalTicks: PPQ, // 1 beat default
                seconds: defaultTimeService.ticksToSeconds(PPQ)
            },
            velocity: 0.8
        }

        updateClip(c.id, { notes: [...c.notes, newNote] })
    }

    const removeNote = (noteId: string) => {
        const c = clip()
        if (!c) return
        updateClip(c.id, { notes: c.notes.filter(n => n.id !== noteId) })
    }

    const handleScroll = () => {
        if (keysContainer && gridContainer) {
            keysContainer.scrollTop = gridContainer.scrollTop
        }
    }

    return (
        <div class='flex h-full w-full bg-surface-container-low overflow-hidden relative'>
            {/* Zoom Controls */}
            <div class='absolute bottom-4 right-4 z-30 flex gap-2 bg-surface-container-high p-1 rounded-full shadow-md border border-outline-variant'>
                <IconButton
                    onClick={() => setZoom(z => Math.max(10, z * 0.8))}
                    variant='standard'
                    class='w-8 h-8'
                >
                    -
                </IconButton>
                <IconButton
                    onClick={() => setZoom(z => Math.min(500, z * 1.2))}
                    variant='standard'
                    class='w-8 h-8'
                >
                    +
                </IconButton>
            </div>

            {/* Keys (Left) */}
            <div
                ref={el => (keysContainer = el)}
                class='w-16 shrink-0 border-r border-outline-variant overflow-hidden bg-white'
            >
                <div class='relative' style={{ height: `${KEYS.length * NOTE_HEIGHT}px` }}>
                    <For each={KEYS}>
                        {note => {
                            const isBlack = [1, 3, 6, 8, 10].includes(note % 12)
                            return (
                                <div
                                    class={`absolute left-0 right-0 border-b border-outline-variant/50 flex items-center justify-end pr-1 text-[10px] ${isBlack ? 'bg-black text-white' : 'bg-white text-black'}`}
                                    style={{
                                        top: `${(127 - note) * NOTE_HEIGHT}px`,
                                        height: `${NOTE_HEIGHT}px`
                                    }}
                                >
                                    {note % 12 === 0 ? `C${Math.floor(note / 12) - 1}` : ''}
                                </div>
                            )
                        }}
                    </For>
                </div>
            </div>

            {/* Grid (Right) */}
            <div
                ref={el => (gridContainer = el)}
                class='flex-1 overflow-auto relative bg-surface-container-lowest cursor-crosshair'
                onScroll={handleScroll}
                onMouseDown={handleGridClick}
            >
                <div
                    class='relative'
                    style={{
                        height: `${KEYS.length * NOTE_HEIGHT}px`,
                        width: `${(clip()?.length.totalTicks || 0) * pixelsPerTick() + 1000}px` // Extra space
                    }}
                >
                    {/* Grid Lines */}
                    <div class='absolute inset-0 pointer-events-none'>
                        {/* Vertical lines for beats */}
                        <For each={Array.from({ length: 100 })}>
                            {(_, i) => (
                                <div
                                    class='absolute top-0 bottom-0 border-l border-outline-variant/20'
                                    style={{ left: `${i() * PPQ * pixelsPerTick()}px` }}
                                ></div>
                            )}
                        </For>
                        {/* Horizontal lines for notes */}
                        <For each={KEYS}>
                            {note => (
                                <div
                                    class='absolute left-0 right-0 border-b border-outline-variant/10'
                                    style={{
                                        top: `${(127 - note) * NOTE_HEIGHT}px`,
                                        height: `${NOTE_HEIGHT}px`
                                    }}
                                ></div>
                            )}
                        </For>
                    </div>

                    {/* Playback Cursor */}
                    <Show when={cursorPosition() !== null}>
                        <div
                            class='absolute top-0 bottom-0 w-[2px] bg-primary z-20 pointer-events-none'
                            style={{ left: `${cursorPosition()}px` }}
                        />
                    </Show>

                    {/* Notes */}
                    <Show when={clip()}>
                        <For each={clip()!.notes}>
                            {note => {
                                const startPx =
                                    defaultTimeService.positionToTicks(note.start) * pixelsPerTick()
                                const widthPx = note.duration.totalTicks * pixelsPerTick()
                                const topPx = (127 - note.note) * NOTE_HEIGHT

                                return (
                                    <div style={{ position: 'absolute', top: `${topPx}px` }}>
                                        <PianoRollNote
                                            note={note.note}
                                            startPx={startPx}
                                            widthPx={widthPx}
                                            heightPx={NOTE_HEIGHT}
                                            velocity={note.velocity}
                                            onUpdate={(s, w) => handleNoteUpdate(note.id, s, w)}
                                            onRemove={() => removeNote(note.id)}
                                        />
                                    </div>
                                )
                            }}
                        </For>
                    </Show>
                </div>
            </div>
        </div>
    )
}
