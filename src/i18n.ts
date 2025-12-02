export type Locale = 'en' | 'zh';

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
        'sidebar.simpleSynth': 'Simple Synth',

        'mixer.label': 'MIXER',
        'mixer.console': 'Mixer Console',

        'tracks.header': 'TRACKS',
        'tracks.addTrack': '+ Add Track',
        'grid.inst': 'Inst...',
        'grid.out': 'Out...',
        'clip.title': 'Clip',
        'icons.mute': 'M',
        'icons.solo': 'S',
        'icons.removeTrack': 'Remove Track',
        'app.toggleTimbre': 'Toggle Timbre Sidebar'
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

        'sidebar.title': '音色',
        'sidebar.label': '标签',
        'sidebar.outputRouting': '输出路由',
        'sidebar.gain': '增益',
        'sidebar.waveform': '波形',
        'sidebar.waveform.sine': '正弦',
        'sidebar.waveform.sqr': '方波',
        'sidebar.waveform.saw': '锯齿波',
        'sidebar.waveform.tri': '三角波',
        'sidebar.remove': '移除',
        'sidebar.addTimbre': '添加音色',
        'sidebar.availablePlugins': '可用插件',
        'sidebar.simpleSynth': '简单合成器',

        'mixer.label': '混音器',
        'mixer.console': '混音台',

        'tracks.header': '轨道',
        'tracks.addTrack': '+ 添加轨道',
        'grid.inst': '乐器...',
        'grid.out': '输出...',
        'clip.title': '片段',
        'icons.mute': '静音',
        'icons.solo': '独奏',
        'icons.removeTrack': '删除轨道',
        'app.toggleTimbre': '切换音色侧栏'
    }
};

let current: Locale = 'zh';

export const setLocale = (locale: Locale) => {
    current = locale;
};

export const t = (key: string) => {
    return translations[current][key] ?? key;
};

export default t;
