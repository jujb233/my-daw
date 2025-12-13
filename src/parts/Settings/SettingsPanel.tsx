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
                                class='flex h-[600px] w-[800px] flex-col overflow-hidden rounded-xl shadow-2xl'
                        >
                                {/* Header */}
                                <div class='border-outline-variant bg-surface-container flex h-16 items-center justify-between border-b px-6'>
                                        <h2 class='text-on-surface text-xl font-semibold'>{t('settings.title')}</h2>
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

                                <div class='flex flex-1 overflow-hidden'>
                                        {/* Sidebar */}
                                        <div class='bg-surface-container-low border-outline-variant flex w-48 flex-col gap-1 border-r p-2'>
                                                <button
                                                        class={`rounded-lg px-4 py-3 text-left text-sm font-medium transition-colors ${
                                                                settingsActiveTab() === 'general'
                                                                        ? 'bg-secondary-container text-on-secondary-container'
                                                                        : 'text-on-surface-variant hover:bg-surface-container-highest'
                                                        }`}
                                                        onClick={() => setSettingsActiveTab('general')}
                                                >
                                                        {t('settings.general')}
                                                </button>
                                                <button
                                                        class={`rounded-lg px-4 py-3 text-left text-sm font-medium transition-colors ${
                                                                settingsActiveTab() === 'audio'
                                                                        ? 'bg-secondary-container text-on-secondary-container'
                                                                        : 'text-on-surface-variant hover:bg-surface-container-highest'
                                                        }`}
                                                        onClick={() => setSettingsActiveTab('audio')}
                                                >
                                                        {t('settings.audio')}
                                                </button>
                                                <button
                                                        class={`rounded-lg px-4 py-3 text-left text-sm font-medium transition-colors ${
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
                                        <div class='bg-surface flex-1 overflow-y-auto p-6'>
                                                <Show when={settingsActiveTab() === 'general'}>
                                                        <div class='flex flex-col gap-6'>
                                                                <section>
                                                                        <h3 class='text-on-surface mb-4 text-lg font-medium'>
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
                                                                        <h3 class='text-on-surface mb-4 text-lg font-medium'>
                                                                                {t('settings.theme')}
                                                                        </h3>
                                                                        <div class='bg-surface-container-highest text-on-surface-variant rounded-lg p-4'>
                                                                                Theme settings coming soon...
                                                                        </div>
                                                                </section>
                                                        </div>
                                                </Show>

                                                <Show when={settingsActiveTab() === 'audio'}>
                                                        <div class='flex flex-col gap-6'>
                                                                <section>
                                                                        <h3 class='text-on-surface mb-4 text-lg font-medium'>
                                                                                {t('settings.audioDevice')}
                                                                        </h3>
                                                                        <div class='bg-surface-container-highest text-on-surface-variant rounded-lg p-4'>
                                                                                Audio device configuration coming
                                                                                soon...
                                                                        </div>
                                                                </section>
                                                        </div>
                                                </Show>

                                                <Show when={settingsActiveTab() === 'plugins'}>
                                                        <div class='flex flex-col gap-6'>
                                                                <div class='flex items-center justify-between'>
                                                                        <h3 class='text-on-surface text-lg font-medium'>
                                                                                {t('settings.plugins')}
                                                                        </h3>
                                                                        <Button
                                                                                onClick={async () => {
                                                                                        try {
                                                                                                const selected =
                                                                                                        await open({
                                                                                                                multiple: false,
                                                                                                                filters: [
                                                                                                                        {
                                                                                                                                name: 'CLAP Plugin',
                                                                                                                                extensions: [
                                                                                                                                        'clap'
                                                                                                                                ]
                                                                                                                        }
                                                                                                                ]
                                                                                                        })

                                                                                                if (selected) {
                                                                                                        await importPlugin(
                                                                                                                selected as string
                                                                                                        )
                                                                                                        await refetch()
                                                                                                }
                                                                                        } catch (e) {
                                                                                                console.error(
                                                                                                        'Failed to import plugin',
                                                                                                        e
                                                                                                )
                                                                                        }
                                                                                }}
                                                                        >
                                                                                {t('sidebar.importPlugin')}
                                                                        </Button>
                                                                </div>

                                                                <div class='flex flex-col gap-2'>
                                                                        <For each={availablePlugins()}>
                                                                                {plugin => (
                                                                                        <div class='bg-surface-container-highest flex items-center justify-between rounded-lg p-3'>
                                                                                                <div class='flex flex-col'>
                                                                                                        <span class='text-on-surface font-medium'>
                                                                                                                {t(
                                                                                                                        `plugins.${plugin.unique_id}`
                                                                                                                ) ===
                                                                                                                `plugins.${plugin.unique_id}`
                                                                                                                        ? plugin.name
                                                                                                                        : t(
                                                                                                                                  `plugins.${plugin.unique_id}`
                                                                                                                          )}
                                                                                                        </span>
                                                                                                        <span class='text-on-surface-variant text-xs'>
                                                                                                                {
                                                                                                                        plugin.vendor
                                                                                                                }
                                                                                                        </span>
                                                                                                </div>
                                                                                                <Button
                                                                                                        variant='text'
                                                                                                        onClick={() => {
                                                                                                                addInstance(
                                                                                                                        plugin.unique_id
                                                                                                                )
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
