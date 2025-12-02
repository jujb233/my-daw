import { Component } from "solid-js";
import { Slider } from "../UI/lib/Slider";
import { updateInstanceParam } from "../store/audio";
import { t } from "../i18n";

interface SimpleSynthProps {
    instId: number;
    params: Record<number, number>;
}

export const SimpleSynth: Component<SimpleSynthProps> = (props) => {
    return (
        <>
            <Slider
                label={t('sidebar.gain')}
                min="0"
                max="1"
                step="0.01"
                value={props.params[10]}
                onInput={(e) => updateInstanceParam(props.instId, 10, parseFloat((e.target as HTMLInputElement).value))}
                valueDisplay={`${Math.round((props.params[10] || 0) * 100)}%`}
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
                            class={`px-1 py-1.5 text-xs font-medium rounded transition-colors ${props.params[11] === w.value
                                ? "bg-primary text-on-primary"
                                : "bg-surface-container-highest text-on-surface-variant hover:bg-surface-container-high"
                                }`}
                            onClick={() => updateInstanceParam(props.instId, 11, w.value)}
                        >
                            {w.label}
                        </button>
                    ))}
                </div>
            </div>
        </>
    );
};
