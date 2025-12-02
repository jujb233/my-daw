import { Component, Show } from "solid-js";
import { Surface } from "../../UI/lib/Surface";
import { Input } from "../../UI/lib/Input";
import { Button } from "../../UI/lib/Button";
import { store, setStore, setBpm, selectClip } from "../../store";
import { PianoRoll } from "../MidiEditor/PianoRoll";
import { IconButton } from "../../UI/lib/IconButton";
import { t } from "../../i18n";

export const BottomEditor: Component = () => {
    return (
        <Surface level={2} class="flex flex-col shrink-0 border-t border-outline-variant transition-all duration-300"
            classList={{
                "h-[300px]": !!store.selectedClipId,
                "h-[80px]": !store.selectedClipId
            }}
        >
            <Show when={store.selectedClipId !== null} fallback={
                <div class="flex items-center px-6 py-4 gap-4 h-full">
                    <div class="flex gap-4 flex-wrap items-end w-full">
                        <Input
                            label={t('bottom.projectName')}
                            value={store.info.name}
                            onInput={(e) => setStore("info", "name", e.currentTarget.value)}
                            class="w-48"
                        />
                        <Input
                            label={t('bottom.artist')}
                            value={store.info.artist}
                            onInput={(e) => setStore("info", "artist", e.currentTarget.value)}
                            class="w-48"
                        />
                        <Input
                            label={t('bottom.bpm')}
                            type="number"
                            value={store.info.bpm}
                            onInput={(e) => setBpm(parseFloat(e.currentTarget.value))}
                            class="w-24"
                        />
                        <Input
                            label={t('bottom.timeSig')}
                            value={`${store.info.timeSignature[0]}/${store.info.timeSignature[1]}`}
                            class="w-24"
                            disabled
                        />
                        <div class="flex-grow"></div>
                        <Button variant="tonal">{t('bottom.saveProject')}</Button>
                        <Button variant="filled">{t('bottom.export')}</Button>
                    </div>
                </div>
            }>
                <div class="flex-1 flex flex-col overflow-hidden">
                    <div class="h-10 border-b border-outline-variant flex items-center justify-between px-4 bg-surface-container">
                        <span class="font-medium text-on-surface">{t('bottom.editor')}</span>
                        <IconButton onClick={() => selectClip(null)} variant="standard">
                            <svg xmlns="http://www.w3.org/2000/svg" height="20" viewBox="0 -960 960 960" width="20" fill="currentColor"><path d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z" /></svg>
                        </IconButton>
                    </div>
                    <div class="flex-1 overflow-hidden">
                        <PianoRoll clipId={store.selectedClipId!} />
                    </div>
                </div>
            </Show>
        </Surface>
    );
};
