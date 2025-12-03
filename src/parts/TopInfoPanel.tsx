import { Component } from 'solid-js'
import { Surface } from '../UI/lib/Surface'
import { store, togglePlayback } from '../store'
import { IconButton } from '../UI/lib/IconButton'
import { t } from '../i18n'
import { openSettings } from '../store/ui'

export const TopInfoPanel: Component = () => {
    return (
        <Surface
            level={2}
            class='h-16 flex items-center px-4 gap-4 md:gap-8 shrink-0 z-10 shadow-sm overflow-x-auto'
        >
            <div class='flex items-center gap-2 mr-4 shrink-0'>
                <IconButton
                    onClick={() => openSettings('general')}
                    class='w-10 h-10'
                    title={t('settings.title')}
                >
                    <svg
                        xmlns='http://www.w3.org/2000/svg'
                        height='24'
                        viewBox='0 -960 960 960'
                        width='24'
                        fill='currentColor'
                    >
                        <path d='m370-80-16-128q-13-5-24.5-12T307-235l-119 50L78-375l103-78q-1-7-1-13.5v-27q0-6.5 1-13.5L78-585l110-190 119 50q11-8 23-15t24-12l16-128h220l16 128q13 5 24.5 12t22.5 15l119-50 110 190-103 78q1 7 1 13.5v27q0 6.5-1 13.5l103 78-110 190-119-50q-11 8-23 15t-24 12l-16 128H370Zm112-260q58 0 99-41t41-99q0-58-41-99t-99-41q-59 0-99.5 41T342-480q0 58 40.5 99t99.5 41Zm0-80q-25 0-42.5-17.5T422-480q0-25 17.5-42.5T482-540q25 0 42.5 17.5T542-480q0 25-17.5 42.5T482-420Z' />
                    </svg>
                </IconButton>
                <IconButton
                    variant='filled'
                    onClick={togglePlayback}
                    class='w-12 h-12 !rounded-full'
                >
                    {store.playback.isPlaying ? (
                        <svg
                            xmlns='http://www.w3.org/2000/svg'
                            height='28'
                            viewBox='0 -960 960 960'
                            width='28'
                            fill='currentColor'
                        >
                            <path d='M520-200v-560h240v560H520Zm-320 0v-560h240v560H200Zm40-80h160v-400H240v400Zm320 0h160v-400H560v400Zm-320-400v400-400Zm320 0v400-400Z' />
                        </svg>
                    ) : (
                        <svg
                            xmlns='http://www.w3.org/2000/svg'
                            height='28'
                            viewBox='0 -960 960 960'
                            width='28'
                            fill='currentColor'
                        >
                            <path d='M320-200v-560l440 280-440 280Zm80-280Zm0 134 210-134-210-134v268Z' />
                        </svg>
                    )}
                </IconButton>
            </div>

            <div class='flex gap-6 md:gap-8 shrink-0'>
                <div class='flex flex-col min-w-[60px]'>
                    <span class='text-xs text-on-surface-variant font-medium'>
                        {t('top.duration')}
                    </span>
                    <span class='text-xl font-mono text-primary whitespace-nowrap'>00:00:00</span>
                </div>
                <div class='flex flex-col min-w-[60px]'>
                    <span class='text-xs text-on-surface-variant font-medium'>{t('top.bars')}</span>
                    <span class='text-xl font-mono text-primary whitespace-nowrap'>
                        {store.playback.currentBar.toFixed(1)}
                    </span>
                </div>
                <div class='flex flex-col min-w-[60px]'>
                    <span class='text-xs text-on-surface-variant font-medium'>{t('top.bpm')}</span>
                    <span class='text-xl font-mono text-primary whitespace-nowrap'>
                        {store.info.bpm.toFixed(2)}
                    </span>
                </div>
                <div class='flex flex-col min-w-[60px]'>
                    <span class='text-xs text-on-surface-variant font-medium'>{t('top.sig')}</span>
                    <span class='text-xl font-mono text-primary whitespace-nowrap'>
                        {store.info.timeSignature[0]}/{store.info.timeSignature[1]}
                    </span>
                </div>
            </div>
        </Surface>
    )
}
