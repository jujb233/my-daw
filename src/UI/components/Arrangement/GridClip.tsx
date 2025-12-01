import { Component, Show } from "solid-js";
import { instances } from "../../../store/audio";
import { mixerTracks } from "../../../store/mixer";

interface GridClipProps {
    name: string;
    color?: string;
    width: number; // in pixels or grid units
    left: number; // in pixels or grid units
    onRemove?: () => void;
    // Add props for editing
    instrumentId?: number;
    targetTrackId?: number;
    onUpdate?: (updates: any) => void;
    onClick?: () => void;
}

export const GridClip: Component<GridClipProps> = (props) => {
    return (
        <div
            class="absolute h-full rounded border border-black/20 overflow-hidden cursor-pointer hover:brightness-110 transition-all group flex flex-col"
            style={{
                width: `${props.width}px`,
                left: `${props.left}px`,
                "background-color": props.color || "#3b82f6"
            }}
            onClick={(e) => {
                e.stopPropagation();
                props.onClick?.();
            }}
        >
            <div class="px-2 py-1 text-xs font-medium text-white truncate select-none flex justify-between items-center">
                <span>{props.name}</span>
            </div>

            {/* Mini Editor (Visible on hover or selection - simplified for now) */}
            <div class="flex-1 p-1 opacity-0 group-hover:opacity-100 transition-opacity flex flex-col gap-1 bg-black/10">
                {/* Instrument Selector */}
                <select
                    class="text-[10px] bg-black/20 text-white border-none rounded px-1 outline-none"
                    onClick={(e) => e.stopPropagation()}
                    value={props.instrumentId ?? ""}
                    onChange={(e) => {
                        const val = e.currentTarget.value;
                        props.onUpdate?.({ instrumentId: val ? parseInt(val) : 0 });
                    }}
                >
                    <option value="">Inst...</option>
                    {instances().map(i => <option value={i.id}>{i.label}</option>)}
                </select>

                {/* Output Selector */}
                <select
                    class="text-[10px] bg-black/20 text-white border-none rounded px-1 outline-none"
                    onClick={(e) => e.stopPropagation()}
                    value={props.targetTrackId ?? ""}
                    onChange={(e) => {
                        const val = e.currentTarget.value;
                        props.onUpdate?.({ targetTrackId: val ? parseInt(val) : 0 });
                    }}
                >
                    <option value="">Out...</option>
                    {mixerTracks().map(t => <option value={t.id}>{t.label}</option>)}
                </select>
            </div>            {/* Remove Button (visible on hover) */}
            {props.onRemove && (
                <div
                    class="absolute top-1 right-1 opacity-0 group-hover:opacity-100 transition-opacity bg-black/50 rounded-full p-0.5 cursor-pointer hover:bg-black/70"
                    onClick={(e) => {
                        e.stopPropagation();
                        props.onRemove?.();
                    }}
                >
                    <svg xmlns="http://www.w3.org/2000/svg" height="12" viewBox="0 -960 960 960" width="12" fill="white"><path d="M280-120q-33 0-56.5-23.5T200-200v-520h-40v-80h200v-40h240v40h200v80h-40v520q0 33-23.5 56.5T680-120H280Zm400-600H280v520h400v-520ZM360-280h80v-360h-80v360Zm160 0h80v-360h-80v360ZM280-720v520-520Z" /></svg>
                </div>
            )}
        </div>
    );
};
