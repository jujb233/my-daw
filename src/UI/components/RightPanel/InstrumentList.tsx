import { Component, For, createSignal, Show } from "solid-js";
import { Button } from "../../lib/Button";
import { Slider } from "../../lib/Slider";
import { instances, addInstance, updateInstanceParam, removeInstance, updateInstanceLabel, toggleInstanceExpanded } from "../../../store/audio";
import { t } from "../../../i18n";
import { InstrumentCard } from "../InstrumentCard";

export const InstrumentList: Component = () => {
    const [showAddMenu, setShowAddMenu] = createSignal(false);

    return (
        <div class="flex-1 overflow-y-auto p-4 flex flex-col gap-3">
            <For each={instances()}>
                {(inst) => (
                    <InstrumentCard
                        label={inst.label}
                        name={inst.name}
                        id={inst.id}
                        isExpanded={inst.isExpanded}
                        onToggleExpand={() => toggleInstanceExpanded(inst.id)}
                        onRemove={() => removeInstance(inst.id)}
                    >
                        {/* Label Editor */}
                        <div>
                            <label class="text-xs text-on-surface-variant block mb-1">{t('sidebar.label')}</label>
                            <input
                                type="text"
                                value={inst.label}
                                onInput={(e) => updateInstanceLabel(inst.id, e.currentTarget.value)}
                                class="w-full bg-surface-container-highest text-on-surface text-sm px-2 py-1 rounded border-none focus:ring-1 focus:ring-primary outline-none"
                            />
                        </div>

                        <Slider
                            label={t('sidebar.gain')}
                            min="0"
                            max="1"
                            step="0.01"
                            value={inst.params[10]}
                            onInput={(e) => updateInstanceParam(inst.id, 10, parseFloat((e.target as HTMLInputElement).value))}
                            valueDisplay={`${Math.round((inst.params[10] || 0) * 100)}%`}
                        />

                        <div class="flex flex-col gap-2">
                            <span class="text-xs text-on-surface-variant">{t('sidebar.waveform')}</span>
                            <div class="grid grid-cols-4 gap-1">
                                {[
                                    { label: t('sidebar.waveform.sine'), value: 0 },
                                    { label: t('sidebar.waveform.sqr'), value: 1 },
                                    { label: t('sidebar.waveform.saw'), value: 2 },
                                    { label: t('sidebar.waveform.tri'), value: 3 },
                                ].map((w) => (
                                    <button
                                        class={`px-1 py-1.5 text-xs font-medium rounded transition-colors ${inst.params[11] === w.value
                                            ? "bg-primary text-on-primary"
                                            : "bg-surface-container-highest text-on-surface-variant hover:bg-surface-container-high"
                                            }`}
                                        onClick={() => updateInstanceParam(inst.id, 11, w.value)}
                                    >
                                        {w.label}
                                    </button>
                                ))}
                            </div>
                        </div>
                    </InstrumentCard>
                )}
            </For>

            <div class="relative mt-2">
                <Button
                    variant="outlined"
                    class="w-full"
                    onClick={() => setShowAddMenu(!showAddMenu())}
                >
                    {t('sidebar.addTimbre')}
                </Button>

                <Show when={showAddMenu()}>
                    <div class="absolute bottom-full left-0 w-full mb-2 bg-surface-container-high rounded-lg shadow-lg border border-outline-variant overflow-hidden z-10">
                        <div class="p-2">
                            <span class="text-xs font-medium text-on-surface-variant px-2">{t('sidebar.availablePlugins')}</span>
                        </div>
                        <button
                            class="w-full text-left px-4 py-2 text-sm text-on-surface hover:bg-surface-container-highest transition-colors flex items-center gap-2"
                            onClick={() => {
                                addInstance("SimpleSynth");
                                setShowAddMenu(false);
                            }}
                        >
                            <span>🎹</span>
                            <span>{t('sidebar.simpleSynth')}</span>
                        </button>
                        {/* Future plugins here */}
                    </div>
                </Show>
            </div>
        </div>
    );
};
