export type Locale = 'en' | 'zh'

const translations: Record<Locale, Record<string, string>> = {
    en: {
        'top.duration': 'DURATION',
        'top.bars': 'BARS',
        'top.bpm': 'BPM',
        'top.sig': 'SIG',

        'bottom.projectName': 'Project Name',
        'bottom.artist': 'Artist',
        'bottom.bpm': 'BPM',
        'bottom.timeSig': 'Time Sig',
        'bottom.saveProject': 'Save Project',
        'bottom.export': 'Export',
        'bottom.midiEditor': 'MIDI Editor',

        'sidebar.title': 'Timbres',
        'sidebar.label': 'Label',
        'sidebar.outputRouting': 'Output Routing',
        'sidebar.gain': 'Gain',
        'sidebar.waveform': 'Waveform',
        'sidebar.waveform.sine': 'Sine',
        'sidebar.waveform.sqr': 'Sqr',
        'sidebar.waveform.saw': 'Saw',
        'sidebar.waveform.tri': 'Tri',
        'sidebar.remove': 'Remove',
        'sidebar.addTimbre': 'Add Timbre',
        'sidebar.availablePlugins': 'Available Plugins',
        'sidebar.importPlugin': 'Import Plugin...',
        'sidebar.simpleSynth': 'Simple Synth',
        'sidebar.instruments': 'Instruments',

        'mixer.label': 'MIXER',
        'mixer.console': 'Mixer Console',

        'tracks.header': 'TRACKS',
        'tracks.addTrack': '+ Add Track',
        'grid.inst': 'Inst...',
        'grid.out': 'Out...',
        'clip.title': 'Clip',
        'clip.name': 'Clip Name',
        'clip.duplicate': 'Duplicate',
        'clip.delete': 'Delete',
        'clip.deleteConfirm': 'Are you sure you want to delete this clip?',
        'icons.mute': 'M',
        'icons.solo': 'S',
        'icons.removeTrack': 'Remove Track',
        'app.toggleTimbre': 'Toggle Timbre Sidebar',
        'settings.title': 'Settings',
        'settings.general': 'General',
        'settings.audio': 'Audio',
        'settings.language': 'Language',
        'settings.theme': 'Theme',
        'settings.audioDevice': 'Audio Device',
        'settings.plugins': 'Plugins'
    },
    zh: {
        'top.duration': '时长',
        'top.bars': '小节',
        'top.bpm': '节拍',
        'top.sig': '拍号',

        'bottom.projectName': '项目名称',
        'bottom.artist': '艺术家',
        'bottom.bpm': 'BPM',
        'bottom.timeSig': '拍号',
        'bottom.saveProject': '保存项目',
        'bottom.export': '导出',
        'bottom.midiEditor': 'MIDI 编辑器',

        'sidebar.title': '音色库',
        'sidebar.label': '标签',
        'sidebar.outputRouting': '输出路由',
        'sidebar.gain': '增益',
        'sidebar.waveform': '波形',
        'sidebar.waveform.sine': '正弦波',
        'sidebar.waveform.sqr': '方波',
        'sidebar.waveform.saw': '锯齿波',
        'sidebar.waveform.tri': '三角波',
        'sidebar.remove': '移除',
        'sidebar.addTimbre': '添加音色',
        'sidebar.availablePlugins': '可用插件',
        'sidebar.importPlugin': '导入插件...',
        'sidebar.simpleSynth': '简单合成器',
        'sidebar.instruments': '乐器',

        'mixer.label': '调音台',
        'mixer.console': '调音台控制台',

        'tracks.header': '轨道',
        'tracks.addTrack': '+ 添加轨道',
        'grid.inst': '乐器...',
        'grid.out': '输出...',
        'clip.title': '片段',
        'clip.name': '片段名称',
        'clip.duplicate': '复制',
        'clip.delete': '删除',
        'clip.deleteConfirm': '确定要删除这个片段吗？',
        'icons.mute': 'M',
        'icons.solo': 'S',
        'icons.removeTrack': '移除轨道',
        'app.toggleTimbre': '切换音色侧边栏',
        'settings.title': '设置',
        'settings.general': '常规',
        'settings.audio': '音频',
        'settings.language': '语言',
        'settings.theme': '主题',
        'settings.audioDevice': '音频设备',
        'settings.plugins': '插件'
    }
}

let current: Locale = (localStorage.getItem('locale') as Locale) || 'zh'

export const setLocale = (locale: Locale) => {
    current = locale
    localStorage.setItem('locale', locale)
}

export const t = (key: string) => {
    return translations[current][key] ?? key
}

export default t
