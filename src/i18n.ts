import { createSignal } from 'solid-js'
import en from './locales/en'
import zh from './locales/zh'

export type Locale = 'en' | 'zh'

const translations: Record<Locale, Record<string, string>> = {
    en,
    zh
}

const getInitialLocale = (): Locale => {
    const saved = localStorage.getItem('locale') as Locale
    if (saved && (saved === 'en' || saved === 'zh')) {
        return saved
    }
    const browserLang = navigator.language
    if (browserLang.startsWith('zh')) {
        return 'zh'
    }
    return 'en'
}

const [locale, setLocaleSignal] = createSignal<Locale>(getInitialLocale())

export const currentLocale = locale

export const setLocale = (l: Locale) => {
    setLocaleSignal(l)
    localStorage.setItem('locale', l)
}

export const t = (key: string) => {
    const l = locale()
    return translations[l][key] ?? key
}

export default t
