import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

export const [masterVolume, setMasterVolume] = createSignal(0.5);

export const sendParameter = async (paramId: number, value: number) => {
    try {
        await invoke("update_parameter", { paramId, value });
    } catch (e) {
        console.error("Failed to update parameter:", e);
    }
};

export const updateMasterVolume = (value: number) => {
    setMasterVolume(value);
    sendParameter(0, value);
};
