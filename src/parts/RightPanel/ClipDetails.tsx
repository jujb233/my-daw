import { Component, createResource, createSignal, For, Show, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { store, selectClip } from "../../store";
import { instances } from "../../store/audio";
import { mixerTracks } from "../../store/mixer";
import { t } from "../../i18n";
import { InstrumentCard } from "../../UI/components/InstrumentCard";

interface ClipData {
    id: number;
    name: string;
    start_time: number;
    duration: number;
    instrument_ids: number[];
    instrument_routes: Record<number, number[]>;
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
    const [expandedInstId, setExpandedInstId] = createSignal<number | null>(null);

    // Handle error by clearing selection
    createEffect(() => {
        if (clip.error) {
            console.warn("Clip fetch failed, clearing selection:", clip.error);
            setTimeout(() => selectClip(null), 0);
        }
    });

    const toggleInstrument = async (instId: number) => {
        if (!clip()) return;
        const currentIds = clip()!.instrument_ids;
        let newIds;
        let newRoutes = { ...clip()!.instrument_routes };

        if (currentIds.includes(instId)) {
            newIds = currentIds.filter(id => id !== instId);
            // Clean up routes
            delete newRoutes[instId];
        } else {
            newIds = [...currentIds, instId];
            // Default route to Master (0)
            newRoutes[instId] = [0];
        }

        // Optimistic update
        mutate({ ...clip()!, instrument_ids: newIds, instrument_routes: newRoutes });

        try {
            await invoke("update_clip", {
                id: clip()!.id,
                instrumentIds: newIds,
                instrumentRoutes: newRoutes
            });
        } catch (e) {
            console.error("Failed to update clip instruments:", e);
            refetch();
        }
    };

    const updateRouting = async (instId: number, trackId: number, add: boolean) => {
        if (!clip()) return;
        const currentRoutes = clip()!.instrument_routes[instId] || [];
        let newRoutesForInst;

        if (add) {
            if (!currentRoutes.includes(trackId)) {
                newRoutesForInst = [...currentRoutes, trackId];
            } else {
                return;
            }
        } else {
            newRoutesForInst = currentRoutes.filter(id => id !== trackId);
        }

        const allRoutes = { ...clip()!.instrument_routes, [instId]: newRoutesForInst };

        // Optimistic update
        mutate({ ...clip()!, instrument_routes: allRoutes });

        try {
            await invoke("update_clip", {
                id: clip()!.id,
                instrumentRoutes: allRoutes
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

                        {/* Active Instruments List */}
                        <div class="flex flex-col gap-3">
                            <span class="text-sm font-medium text-on-surface">{t('sidebar.instruments')}</span>

                            {/* List of selected instruments as Cards */}
                            <For each={clip()!.instrument_ids}>
                                {(instId) => {
                                    const inst = instances().find(i => i.id === instId);
                                    if (!inst) return null;

                                    return (
                                        <InstrumentCard
                                            label={inst.label}
                                            name={inst.name}
                                            id={inst.id}
                                            isExpanded={expandedInstId() === inst.id}
                                            onToggleExpand={() => setExpandedInstId(expandedInstId() === inst.id ? null : inst.id)}
                                            onRemove={() => toggleInstrument(inst.id)}
                                        >
                                            <div class="flex flex-col gap-2">
                                                <span class="text-xs text-on-surface-variant">{t('sidebar.outputRouting')}</span>
                                                <div class="flex flex-col gap-1">
                                                    <For each={mixerTracks()}>
                                                        {(track) => (
                                                            <label class="flex items-center gap-2 p-2 rounded hover:bg-surface-container-high cursor-pointer">
                                                                <input
                                                                    type="checkbox"
                                                                    checked={(clip()!.instrument_routes[inst.id] || []).includes(track.id)}
                                                                    onChange={(e) => updateRouting(inst.id, track.id, e.currentTarget.checked)}
                                                                    class="w-4 h-4 accent-primary"
                                                                />
                                                                <span class="text-sm text-on-surface">{track.label}</span>
                                                            </label>
                                                        )}
                                                    </For>
                                                </div>
                                            </div>
                                        </InstrumentCard>
                                    );
                                }}
                            </For>

                            {/* Add Instrument Button */}
                            <div class="relative mt-2">
                                <span class="text-xs text-on-surface-variant block mb-2">Add Instrument to Clip:</span>
                                <div class="flex flex-col gap-1">
                                    <For each={instances().filter(i => !clip()!.instrument_ids.includes(i.id))}>
                                        {(inst) => (
                                            <button
                                                class="flex items-center gap-2 p-2 rounded hover:bg-surface-container-high text-left transition-colors border border-transparent hover:border-outline-variant"
                                                onClick={() => toggleInstrument(inst.id)}
                                            >
                                                <div class="w-6 h-6 rounded bg-secondary/20 flex items-center justify-center text-secondary text-xs">
                                                    +
                                                </div>
                                                <span class="text-sm text-on-surface">{inst.label}</span>
                                            </button>
                                        )}
                                    </For>
                                    <Show when={instances().length === 0}>
                                        <span class="text-xs text-on-surface-variant italic">No instruments available</span>
                                    </Show>
                                </div>
                            </div>
                        </div>
                    </Show>
                </Show>
            </Show>
        </div>
    );
};
