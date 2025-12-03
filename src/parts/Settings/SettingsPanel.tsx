import { Component, Show, createResource, For } from 'solid-js'
import { open } from '@tauri-apps/plugin-dialog'
import { Surface } from '../../UI/lib/Surface'
import { Button } from '../../UI/lib/Button'
import { IconButton } from '../../UI/lib/IconButton'
import { t, setLocale } from '../../i18n'
import { settingsActiveTab, setSettingsActiveTab } from '../../store/ui'
import { getAvailablePlugins, importPlugin } from '../../services/plugin'
import { addInstance } from '../../store/audio'

interface SettingsPanelProps {
    onClose: () => void
}

export const SettingsPanel: Component<SettingsPanelProps> = props => {
    const [availablePlugins, { refetch }] = createResource(getAvailablePlugins)

    return (
        <div class='fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm'>
            <Surface
                level={1}
                class='w-[800px] h-[600px] flex flex-col rounded-xl shadow-2xl overflow-hidden'
            >
                {/* Header */}
                <div class='h-16 px-6 flex items-center justify-between border-b border-outline-variant bg-surface-container'>
                    <h2 class='text-xl font-semibold text-on-surface'>{t('settings.title')}</h2>
                    <IconButton onClick={props.onClose}>
                        <svg
                            xmlns='http://www.w3.org/2000/svg'
                            height='24'
                            viewBox='0 -960 960 960'
                            width='24'
                            fill='currentColor'
                        >
                            <path d='m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z' />
                        </svg>
                    </IconButton>
                </div>

                <div class='flex-1 flex overflow-hidden'>
                    {/* Sidebar */}
                    <div class='w-48 bg-surface-container-low border-r border-outline-variant p-2 flex flex-col gap-1'>
                        <button
                            class={`px-4 py-3 text-left rounded-lg text-sm font-medium transition-colors ${
                                settingsActiveTab() === 'general'
                                    ? 'bg-secondary-container text-on-secondary-container'
                                    : 'text-on-surface-variant hover:bg-surface-container-highest'
                            }`}
                            onClick={() => setSettingsActiveTab('general')}
                        >
                            {t('settings.general')}
                        </button>
                        <button
                            class={`px-4 py-3 text-left rounded-lg text-sm font-medium transition-colors ${
                                settingsActiveTab() === 'audio'
                                    ? 'bg-secondary-container text-on-secondary-container'
                                    : 'text-on-surface-variant hover:bg-surface-container-highest'
                            }`}
                            onClick={() => setSettingsActiveTab('audio')}
                        >
                            {t('settings.audio')}
                        </button>
                        <button
                            class={`px-4 py-3 text-left rounded-lg text-sm font-medium transition-colors ${
                                settingsActiveTab() === 'plugins'
                                    ? 'bg-secondary-container text-on-secondary-container'
                                    : 'text-on-surface-variant hover:bg-surface-container-highest'
                            }`}
                            onClick={() => setSettingsActiveTab('plugins')}
                        >
                            {t('settings.plugins')}
                        </button>
                    </div>

                    {/* Content */}
                    <div class='flex-1 p-6 overflow-y-auto bg-surface'>
                        <Show when={settingsActiveTab() === 'general'}>
                            <div class='flex flex-col gap-6'>
                                <section>
                                    <h3 class='text-lg font-medium text-on-surface mb-4'>
                                        {t('settings.language')}
                                    </h3>
                                    <div class='flex gap-2'>
                                        <Button
                                            variant='outlined'
                                            onClick={() => {
                                                setLocale('en')
                                            }}
                                        >
                                            English
                                        </Button>
                                        <Button
                                            variant='outlined'
                                            onClick={() => {
                                                setLocale('zh')
                                            }}
                                        >
                                            中文
                                        </Button>
                                    </div>
                                </section>
                                <section>
                                    <h3 class='text-lg font-medium text-on-surface mb-4'>
                                        {t('settings.theme')}
                                    </h3>
                                    <div class='p-4 rounded-lg bg-surface-container-highest text-on-surface-variant'>
                                        Theme settings coming soon...
                                    </div>
                                </section>
                            </div>
                        </Show>

                        <Show when={settingsActiveTab() === 'audio'}>
                            <div class='flex flex-col gap-6'>
                                <section>
                                    <h3 class='text-lg font-medium text-on-surface mb-4'>
                                        {t('settings.audioDevice')}
                                    </h3>
                                    <div class='p-4 rounded-lg bg-surface-container-highest text-on-surface-variant'>
                                        Audio device configuration coming soon...
                                    </div>
                                </section>
                            </div>
                        </Show>

                        <Show when={settingsActiveTab() === 'plugins'}>
                            <div class='flex flex-col gap-6'>
                                <div class='flex justify-between items-center'>
                                    <h3 class='text-lg font-medium text-on-surface'>
                                        {t('settings.plugins')}
                                    </h3>
                                    <Button
                                        onClick={async () => {
                                            try {
                                                const selected = await open({
                                                    multiple: false,
                                                    filters: [
                                                        {
                                                            name: 'CLAP Plugin',
                                                            extensions: ['clap']
                                                        }
                                                    ]
                                                })

                                                if (selected) {
                                                    await importPlugin(selected as string)
                                                    await refetch()
                                                }
                                            } catch (e) {
                                                console.error('Failed to import plugin', e)
                                            }
                                        }}
                                    >
                                        {t('sidebar.importPlugin')}
                                    </Button>
                                </div>

                                <div class='flex flex-col gap-2'>
                                    <For each={availablePlugins()}>
                                        {plugin => (
                                            <div class='flex items-center justify-between p-3 bg-surface-container-highest rounded-lg'>
                                                <div class='flex flex-col'>
                                                    <span class='font-medium text-on-surface'>
                                                        {t(`plugins.${plugin.unique_id}`) ===
                                                        `plugins.${plugin.unique_id}`
                                                            ? plugin.name
                                                            : t(`plugins.${plugin.unique_id}`)}
                                                    </span>
                                                    <span class='text-xs text-on-surface-variant'>
                                                        {plugin.vendor}
                                                    </span>
                                                </div>
                                                <Button
                                                    variant='text'
                                                    onClick={() => {
                                                        addInstance(plugin.unique_id)
                                                        props.onClose()
                                                    }}
                                                >
                                                    Add
                                                </Button>
                                            </div>
                                        )}
                                    </For>
                                </div>
                            </div>
                        </Show>
                    </div>
                </div>
            </Surface>
        </div>
    )
}
