import { Component, createResource, createSignal, For, Show, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { store, selectClip } from "../../../store";
import { instances } from "../../../store/audio";
import { mixerTracks } from "../../../store/mixer";
import { t } from "../../../i18n";
import { Surface } from "../../lib/Surface";

interface ClipData {
    id: number;
    name: string;
    start_time: number;
    duration: number;
    instrument_ids: number[];
    target_track_ids: number[];
}

const fetchClip = async (id: number): Promise<ClipData> => {
    try {
        return await invoke("get_clip", { id });
    } catch (e) {
        throw new Error("Clip not found");
    }
};

export const ClipDetails: Component = () => {
    const [clip, { mutate, refetch }] = createResource(() => store.selectedClipId, fetchClip);

    // Handle error by clearing selection
    createEffect(() => {
        if (clip.error) {
            console.warn("Clip fetch failed, clearing selection:", clip.error);
            // Use a timeout to ensure we don't conflict with render cycles
            setTimeout(() => selectClip(null), 0);
        }
    });

    const toggleInstrument = async (instId: number) => {
        if (!clip()) return;
        const currentIds = clip()!.instrument_ids;
        let newIds;
        if (currentIds.includes(instId)) {
            newIds = currentIds.filter(id => id !== instId);
        } else {
            newIds = [...currentIds, instId];
        }

        // Optimistic update
        mutate({ ...clip()!, instrument_ids: newIds });

        try {
            await invoke("update_clip", {
                id: clip()!.id,
                instrumentIds: newIds
            });
        } catch (e) {
            console.error("Failed to update clip instruments:", e);
            refetch(); // Revert on error
        }
    };

    const toggleTargetTrack = async (trackId: number) => {
        if (!clip()) return;
        const currentIds = clip()!.target_track_ids;
        let newIds;
        if (currentIds.includes(trackId)) {
            newIds = currentIds.filter(id => id !== trackId);
        } else {
            newIds = [...currentIds, trackId];
        }

        // Optimistic update
        mutate({ ...clip()!, target_track_ids: newIds });

        try {
            await invoke("update_clip", {
                id: clip()!.id,
                targetTrackIds: newIds
            });
        } catch (e) {
            console.error("Failed to update clip routing:", e);
            refetch();
        }
    };

    return (
        <div class="flex-1 overflow-y-auto p-4 flex flex-col gap-4">
            <Show when={!clip.loading} fallback={<div class="text-center text-on-surface-variant p-4">Loading...</div>}>
                <Show when={!clip.error} fallback={<div class="text-center text-error p-4">Error loading clip</div>}>
                    <Show when={clip()}>
                        <div class="flex flex-col gap-1 pb-2 border-b border-outline-variant">
                            <span class="text-lg font-medium text-on-surface">{clip()!.name}</span>
                            <span class="text-xs text-on-surface-variant">ID: {clip()!.id}</span>
                        </div>

                        {/* Instruments Selection */}
                        <Surface level={1} class="flex flex-col p-3 gap-2">
                            <span class="text-sm font-medium text-on-surface">{t('sidebar.instruments')}</span>
                            <div class="flex flex-col gap-1">
                                <For each={instances()}>
                                    {(inst) => (
                                        <label class="flex items-center gap-2 p-2 rounded hover:bg-surface-container-high cursor-pointer">
                                            <input
                                                type="checkbox"
                                                checked={clip()!.instrument_ids.includes(inst.id)}
                                                onChange={() => toggleInstrument(inst.id)}
                                                class="w-4 h-4 accent-primary"
                                            />
                                            <span class="text-sm text-on-surface">{inst.label}</span>
                                        </label>
                                    )}
                                </For>
                                <Show when={instances().length === 0}>
                                    <span class="text-xs text-on-surface-variant italic p-2">No instruments available</span>
                                </Show>
                            </div>
                        </Surface>

                        {/* Routing Selection */}
                        <Surface level={1} class="flex flex-col p-3 gap-2">
                            <span class="text-sm font-medium text-on-surface">{t('sidebar.routing')}</span>
                            <div class="flex flex-col gap-1">
                                <For each={mixerTracks()}>
                                    {(track) => (
                                        <label class="flex items-center gap-2 p-2 rounded hover:bg-surface-container-high cursor-pointer">
                                            <input
                                                type="checkbox"
                                                checked={clip()!.target_track_ids.includes(track.id)}
                                                onChange={() => toggleTargetTrack(track.id)}
                                                class="w-4 h-4 accent-primary"
                                            />
                                            <span class="text-sm text-on-surface">{track.label}</span>
                                        </label>
                                    )}
                                </For>
                            </div>
                        </Surface>
                    </Show>
                </Show>
            </Show>
        </div>
    );
};
