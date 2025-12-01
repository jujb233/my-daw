import { Component } from "solid-js";
import { MixerTrackData, setTrackVolume, removeMixerTrack } from "../../../store/mixer";
import { Fader } from "../../lib/Fader";
import { LevelMeter } from "../../lib/LevelMeter";
import { IconButton } from "../../lib/IconButton";

interface MixerStripProps {
    track: MixerTrackData;
    level: number; // 0-1
}

export const MixerStrip: Component<MixerStripProps> = (props) => {
    return (
        <div class="w-24 bg-surface-container-low border-r border-outline-variant flex flex-col items-center py-2 gap-2 shrink-0">
            {/* Header / Label */}
            <div class="w-full px-2 text-center">
                <span class="text-xs font-medium text-on-surface truncate block" title={props.track.label}>
                    {props.track.label}
                </span>
            </div>

            {/* Pan / Other controls placeholder */}
            <div class="w-8 h-8 rounded-full border border-outline-variant flex items-center justify-center">
                <div class="w-0.5 h-3 bg-on-surface-variant rotate-0" />
            </div>

            {/* Meter & Fader Container */}
            <div class="flex-1 flex gap-2 items-end justify-center w-full px-2 py-2 bg-surface-container-lowest rounded mx-1">
                <LevelMeter level={props.level} className="h-48 w-3" />
                <Fader
                    value={props.track.volume}
                    onChange={(v) => setTrackVolume(props.track.id, v)}
                    className="h-48"
                />
            </div>

            {/* Mute / Solo / Rec */}
            <div class="flex gap-1">
                <button class="w-6 h-6 text-[10px] font-bold bg-surface-container-highest text-on-surface-variant rounded hover:bg-surface-container-high">M</button>
                <button class="w-6 h-6 text-[10px] font-bold bg-surface-container-highest text-on-surface-variant rounded hover:bg-surface-container-high">S</button>
            </div>

            {/* Delete Button (if not master, maybe?) */}
            <IconButton
                onClick={() => removeMixerTrack(props.track.id)}
                variant="standard"
                class="text-error hover:bg-error/10"
            >
                <svg xmlns="http://www.w3.org/2000/svg" height="18" viewBox="0 -960 960 960" width="18" fill="currentColor"><path d="M280-120q-33 0-56.5-23.5T200-200v-520h-40v-80h200v-40h240v40h200v80h-40v520q0 33-23.5 56.5T680-120H280Zm400-600H280v520h400v-520ZM360-280h80v-360h-80v360Zm160 0h80v-360h-80v360ZM280-720v520-520Z" /></svg>
            </IconButton>
        </div>
    );
};
