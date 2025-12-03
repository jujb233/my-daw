import { Component } from 'solid-js'
import { Surface } from '../UI/lib/Surface'
import { store, togglePlayback } from '../store'
import { IconButton } from '../UI/lib/IconButton'
import { t } from '../i18n'

export const TopInfoPanel: Component = () => {
    return (
        <Surface
            level={2}
            class='h-16 flex items-center px-4 gap-4 md:gap-8 shrink-0 z-10 shadow-sm overflow-x-auto'
        >
            <div class='flex items-center gap-2 mr-4 shrink-0'>
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
