import { createSignal } from 'solid-js'

export const [showSettings, setShowSettings] = createSignal(false)
export const [settingsActiveTab, setSettingsActiveTab] = createSignal('general')

export const openSettings = (tab: string = 'general') => {
        setSettingsActiveTab(tab)
        setShowSettings(true)
}
