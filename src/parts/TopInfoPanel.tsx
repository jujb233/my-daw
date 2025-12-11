import { Component } from 'solid-js'
import { Surface } from '../UI/lib/Surface'
import { store, togglePlayback, setSnapInterval } from '../store'
import { SnapGrid } from '../services/time'
import { IconButton } from '../UI/lib/IconButton'
import { t } from '../i18n'
import { openSettings } from '../store/ui'

export const TopInfoPanel: Component = () => {
    return (
        <Surface
            level={2}
            class='z-10 flex h-16 shrink-0 items-center gap-4 overflow-x-auto px-4 shadow-sm md:gap-8'
        >
            <div class='mr-4 flex shrink-0 items-center gap-2'>
                <IconButton
                    onClick={() => openSettings('general')}
                    class='h-10 w-10'
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

                {/* Snap Selector */}
                <div class='border-outline-variant flex items-center gap-2 rounded border px-2 py-1'>
                    <span class='text-on-surface-variant text-xs'>Snap:</span>
                    <select
                        class='text-on-surface bg-transparent text-sm outline-none'
                        value={store.snapInterval}
                        onChange={e => setSnapInterval(e.currentTarget.value as SnapGrid)}
                    >
                        <option value='1/1'>1/1</option>
                        <option value='1/2'>1/2</option>
                        <option value='1/4'>1/4</option>
                        <option value='1/8'>1/8</option>
                        <option value='1/16'>1/16</option>
                        <option value='1/32'>1/32</option>
                    </select>
                </div>

                <IconButton
                    variant='filled'
                    onClick={togglePlayback}
                    class='h-12 w-12 !rounded-full'
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

            <div class='flex shrink-0 gap-6 md:gap-8'>
                <div class='flex min-w-[60px] flex-col'>
                    <span class='text-on-surface-variant text-xs font-medium'>
                        {t('top.duration')}
                    </span>
                    <span class='text-primary font-mono text-xl whitespace-nowrap'>00:00:00</span>
                </div>
                <div class='flex min-w-[60px] flex-col'>
                    <span class='text-on-surface-variant text-xs font-medium'>{t('top.bars')}</span>
                    <span class='text-primary font-mono text-xl whitespace-nowrap'>
                        {store.playback.currentPosition.bar}.{store.playback.currentPosition.beat}
                    </span>
                </div>
                <div class='flex min-w-[60px] flex-col'>
                    <span class='text-on-surface-variant text-xs font-medium'>{t('top.bpm')}</span>
                    <span class='text-primary font-mono text-xl whitespace-nowrap'>
                        {store.info.bpm.toFixed(2)}
                    </span>
                </div>
                <div class='flex min-w-[60px] flex-col'>
                    <span class='text-on-surface-variant text-xs font-medium'>{t('top.sig')}</span>
                    <span class='text-primary font-mono text-xl whitespace-nowrap'>
                        {store.info.timeSignature.numerator}/{store.info.timeSignature.denominator}
                    </span>
                </div>
            </div>
        </Surface>
    )
}
