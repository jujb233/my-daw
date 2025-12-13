import { Position, TimeSignature } from '../store/model'

export const PPQ = 960

export type SnapGrid = '1/1' | '1/2' | '1/4' | '1/8' | '1/16' | '1/32'

export class TimeService {
        private bpm: number
        private timeSignature: TimeSignature

        constructor(bpm: number = 120, timeSignature: TimeSignature = { numerator: 4, denominator: 4 }) {
                this.bpm = bpm
                this.timeSignature = timeSignature
        }

        setBpm(bpm: number) {
                this.bpm = bpm
        }

        setTimeSignature(ts: TimeSignature) {
                this.timeSignature = ts
        }

        // --- Conversions ---

        ticksToSeconds(ticks: number): number {
                const secondsPerBeat = 60.0 / this.bpm
                const secondsPerTick = secondsPerBeat / PPQ
                return ticks * secondsPerTick
        }

        secondsToTicks(seconds: number): number {
                const secondsPerBeat = 60.0 / this.bpm
                const secondsPerTick = secondsPerBeat / PPQ
                return Math.round(seconds / secondsPerTick)
        }

        ticksToPosition(totalTicks: number): Position {
                const ticksPerBeat = PPQ
                const beatsPerBar = this.timeSignature.numerator
                const ticksPerBar = ticksPerBeat * beatsPerBar

                const bar = Math.floor(totalTicks / ticksPerBar) + 1
                const remainderAfterBar = totalTicks % ticksPerBar

                const beat = Math.floor(remainderAfterBar / ticksPerBeat) + 1
                const remainderAfterBeat = remainderAfterBar % ticksPerBeat

                const ticksPerSixteenth = PPQ / 4
                const sixteenth = Math.floor(remainderAfterBeat / ticksPerSixteenth) + 1
                const tick = remainderAfterBeat % ticksPerSixteenth

                return {
                        bar,
                        beat,
                        sixteenth,
                        tick,
                        time: this.ticksToSeconds(totalTicks)
                }
        }

        positionToTicks(pos: Position): number {
                const ticksPerBeat = PPQ
                const beatsPerBar = this.timeSignature.numerator
                const ticksPerBar = ticksPerBeat * beatsPerBar
                const ticksPerSixteenth = PPQ / 4

                // Note: Input is 1-based, so subtract 1
                return (
                        (pos.bar - 1) * ticksPerBar +
                        (pos.beat - 1) * ticksPerBeat +
                        (pos.sixteenth - 1) * ticksPerSixteenth +
                        pos.tick
                )
        }

        // --- Snapping ---

        getGridTicks(grid: SnapGrid): number {
                switch (grid) {
                        case '1/1':
                                return PPQ * 4 // Assuming 4/4 for now, strictly this depends on TS denominator
                        case '1/2':
                                return PPQ * 2
                        case '1/4':
                                return PPQ
                        case '1/8':
                                return PPQ / 2
                        case '1/16':
                                return PPQ / 4
                        case '1/32':
                                return PPQ / 8
                        default:
                                return PPQ
                }
        }

        snapTicks(ticks: number, grid: SnapGrid): number {
                const gridTicks = this.getGridTicks(grid)
                return Math.round(ticks / gridTicks) * gridTicks
        }

        snapPosition(pos: Position, grid: SnapGrid): Position {
                const ticks = this.positionToTicks(pos)
                const snappedTicks = this.snapTicks(ticks, grid)
                return this.ticksToPosition(snappedTicks)
        }

        // Helper to create a Position from just a bar number (start of bar)
        fromBar(bar: number): Position {
                return this.ticksToPosition((bar - 1) * (PPQ * this.timeSignature.numerator))
        }
}

export const defaultTimeService = new TimeService()
