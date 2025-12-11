import { createSignal, Show, onMount } from 'solid-js'
import { TopInfoPanel } from './parts/TopInfoPanel'
import { BottomEditor } from './parts/BottomEditor/BottomEditor'
import { TrackEditor } from './parts/Arrangement/TrackEditor'
import { RightPanel } from './parts/RightPanel/RightPanel'
import { MixerPanel } from './parts/Mixer/MixerPanel'
import { SettingsPanel } from './parts/Settings/SettingsPanel'
import { IconButton } from './UI/lib/IconButton'
import { t } from './i18n'
import { showSettings, setShowSettings } from './store/ui'
import { fetchTracks } from './store'
import { fetchMixerTracks } from './store/mixer'
import { fetchInstances } from './store/audio'

export default function App() {
    const [isSidebarOpen, setIsSidebarOpen] = createSignal(true)

    onMount(() => {
        fetchInstances()
        fetchMixerTracks()
        fetchTracks()
    })

    return (
        <div class='bg-background text-on-background flex h-screen w-screen flex-col overflow-hidden'>
            {/* Top Bar */}
            <div class='bg-surface-container-high border-outline-variant relative z-30 flex items-center border-b pr-4'>
                <div class='flex-1'>
                    <TopInfoPanel />
                </div>
                <div class='shrink-0 pl-4'>
                    <IconButton
                        variant={isSidebarOpen() ? 'filled' : 'standard'}
                        onClick={() => setIsSidebarOpen(!isSidebarOpen())}
                        title={t('app.toggleTimbre')}
                        class='h-12 w-12'
                    >
                        <svg
                            xmlns='http://www.w3.org/2000/svg'
                            height='24'
                            viewBox='0 -960 960 960'
                            width='24'
                            fill='currentColor'
                        >
                            <path d='M120-240v-80h720v80H120Zm0-200v-80h720v80H120Zm0-200v-80h720v80H120Z' />
                        </svg>
                    </IconButton>
                </div>
            </div>

            {/* Main Content */}
            <div class='relative flex flex-1 overflow-hidden'>
                {/* Mixer Panel (Left, Collapsible) */}
                <MixerPanel />

                {/* Timeline (Center) */}
                <div class='relative z-0 flex min-w-0 flex-1 flex-col'>
                    <TrackEditor />
                </div>

                {/* Right Panel (Collapsible) */}
                <RightPanel isOpen={isSidebarOpen()} onClose={() => setIsSidebarOpen(false)} />
            </div>

            {/* Bottom Bar */}
            <BottomEditor />
            {/* Settings Modal */}
            <Show when={showSettings()}>
                <SettingsPanel onClose={() => setShowSettings(false)} />
            </Show>
        </div>
    )
}
