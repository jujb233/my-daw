import { Component, Show } from "solid-js";
import { IconButton } from "../../UI/lib/IconButton";
import { t } from "../../i18n";
import { store } from "../../store";
import { InstrumentList } from "./InstrumentList";
import { ClipDetails } from "./ClipDetails";

export const RightPanel: Component<{ isOpen: boolean; onClose: () => void }> = (props) => {
    return (
        <div
            class={`transition-all duration-300 ease-in-out overflow-hidden flex flex-col border-l border-outline-variant bg-surface-container-low ${props.isOpen ? "w-80 opacity-100" : "w-0 opacity-0"
                }`}
        >
            <div class="h-14 flex items-center justify-between px-4 border-b border-outline-variant shrink-0">
                <span class="font-medium text-on-surface">
                    {store.selectedClipId !== null ? t('sidebar.clipDetails') : t('sidebar.title')}
                </span>
                <IconButton onClick={props.onClose} variant="standard">
                    <svg xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 -960 960 960" width="24" fill="currentColor"><path d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z" /></svg>
                </IconButton>
            </div>

            <Show when={store.selectedClipId !== null} fallback={<InstrumentList />}>
                <ClipDetails />
            </Show>
        </div>
    );
};
