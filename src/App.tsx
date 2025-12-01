import { createSignal } from "solid-js";
import { TopInfoPanel } from "./UI/components/TopInfoPanel";
import { BottomEditor } from "./UI/components/BottomEditor";
import { Timeline } from "./UI/components/Arrangement/Timeline";
import { TimbreSidebar } from "./UI/components/TimbreSidebar";
import { MixerPanel } from "./UI/components/Mixer/MixerPanel";
import { IconButton } from "./UI/lib/IconButton";

export default function App() {
  const [isSidebarOpen, setIsSidebarOpen] = createSignal(true);

  return (
    <div class="flex flex-col h-screen w-screen bg-background text-on-background overflow-hidden">
      {/* Top Bar */}
      <div class="flex items-center pr-4 bg-surface-container-high border-b border-outline-variant z-30 relative">
        <div class="flex-1">
          <TopInfoPanel />
        </div>
        <div class="shrink-0 pl-4">
          <IconButton
            variant={isSidebarOpen() ? "filled" : "standard"}
            onClick={() => setIsSidebarOpen(!isSidebarOpen())}
            title="Toggle Timbre Sidebar"
          >
            <svg xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 -960 960 960" width="24" fill="currentColor"><path d="M120-240v-80h720v80H120Zm0-200v-80h720v80H120Zm0-200v-80h720v80H120Z" /></svg>
          </IconButton>
        </div>
      </div>

      {/* Main Content */}
      <div class="flex-1 flex overflow-hidden relative">
        {/* Mixer Panel (Left, Collapsible) */}
        <MixerPanel />

        {/* Timeline (Center) */}
        <div class="flex-1 flex flex-col min-w-0 ml-12"> {/* ml-12 to account for collapsed mixer */}
          <Timeline />
        </div>

        {/* Timbre Sidebar (Right, Collapsible) */}
        <TimbreSidebar isOpen={isSidebarOpen()} onClose={() => setIsSidebarOpen(false)} />
      </div>

      {/* Bottom Bar */}
      <BottomEditor />
    </div>
  );
}
