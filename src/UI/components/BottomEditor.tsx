import { Component } from "solid-js";
import { Surface } from "../lib/Surface";
import { Input } from "../lib/Input";
import { Button } from "../lib/Button";
import { store, setStore, setBpm } from "../../store";

export const BottomEditor: Component = () => {
  return (
    <Surface level={2} class="h-auto min-h-[80px] flex items-center px-6 py-4 gap-4 shrink-0 border-t border-outline-variant">
      <div class="flex gap-4 flex-wrap items-end w-full">
        <Input 
            label="Project Name" 
            value={store.info.name} 
            onInput={(e) => setStore("info", "name", e.currentTarget.value)}
            class="w-48" 
        />
        <Input 
            label="Artist" 
            value={store.info.artist} 
            onInput={(e) => setStore("info", "artist", e.currentTarget.value)}
            class="w-48" 
        />
        <Input 
            label="BPM" 
            type="number" 
            value={store.info.bpm} 
            onInput={(e) => setBpm(parseFloat(e.currentTarget.value))}
            class="w-24" 
        />
        <Input 
            label="Time Sig" 
            value={`${store.info.timeSignature[0]}/${store.info.timeSignature[1]}`} 
            class="w-24" 
            disabled
        />
        <div class="flex-grow"></div>
        <Button variant="tonal">Save Project</Button>
        <Button variant="filled">Export</Button>
      </div>
    </Surface>
  );
};
