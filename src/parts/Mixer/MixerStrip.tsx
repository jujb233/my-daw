import { Component } from 'solid-js'
import {
    MixerTrackData,
    setTrackVolume,
    removeMixerTrack,
    toggleMute,
    toggleSolo
} from '../../store/mixer'
import { Fader } from '../../UI/lib/Fader'
import { LevelMeter } from '../../UI/lib/LevelMeter'
import { IconButton } from '../../UI/lib/IconButton'
import { t } from '../../i18n'

interface MixerStripProps {
    track: MixerTrackData
    level: number // 0-1
}

export const MixerStrip: Component<MixerStripProps> = props => {
    return (
        <div class='w-28 bg-surface-container-low border-r border-outline-variant flex flex-col items-center py-3 gap-3 shrink-0 select-none'>
            {/* Header / Label */}
            <div class='w-full px-2 text-center'>
                <span
                    class='text-sm font-medium text-on-surface truncate block'
                    title={props.track.label}
                >
                    {props.track.label}
                </span>
            </div>

            {/* Pan / Other controls placeholder */}
            {/* Made larger for touch */}
            <div class='w-10 h-10 rounded-full border border-outline-variant flex items-center justify-center bg-surface hover:bg-surface-container-high cursor-pointer'>
                <div class='w-0.5 h-4 bg-primary rotate-0' />
            </div>

            {/* Meter & Fader Container */}
            <div class='flex-1 flex gap-3 items-end justify-center w-full px-2 py-2 bg-surface-container-lowest rounded-lg mx-1 shadow-inner'>
                <LevelMeter level={props.level} className='h-64 w-4' />
                <Fader
                    value={props.track.volume}
                    onChange={v => setTrackVolume(props.track.id, v)}
                    className='h-64 w-12'
                    defaultValue={0.75}
                />
            </div>

            {/* Mute / Solo / Rec */}
            <div class='flex gap-2 w-full justify-center px-2'>
                <button
                    title={t('icons.mute')}
                    class={`flex-1 h-10 text-xs font-bold rounded transition-colors ${
                        props.track.mute
                            ? 'bg-error text-on-error'
                            : 'bg-surface-container-highest text-on-surface-variant hover:bg-surface-container-high'
                    }`}
                    onClick={() => toggleMute(props.track.id)}
                >
                    M
                </button>
                <button
                    title={t('icons.solo')}
                    class={`flex-1 h-10 text-xs font-bold rounded transition-colors ${
                        props.track.solo
                            ? 'bg-tertiary text-on-tertiary'
                            : 'bg-surface-container-highest text-on-surface-variant hover:bg-surface-container-high'
                    }`}
                    onClick={() => toggleSolo(props.track.id)}
                >
                    S
                </button>
            </div>

            {/* Delete Button */}
            {props.track.id !== 0 && (
                <IconButton
                    onClick={() => removeMixerTrack(props.track.id)}
                    variant='standard'
                    class='text-error hover:bg-error/10 w-10 h-10'
                    title={t('icons.removeTrack')}
                >
                    <svg
                        xmlns='http://www.w3.org/2000/svg'
                        height='24'
                        viewBox='0 -960 960 960'
                        width='24'
                        fill='currentColor'
                    >
                        <path d='M280-120q-33 0-56.5-23.5T200-200v-520h-40v-80h200v-40h240v40h200v80h-40v520q0 33-23.5 56.5T680-120H280Zm400-600H280v520h400v-520ZM360-280h80v-360h-80v360Zm160 0h80v-360h-80v360ZM280-720v520-520Z' />
                    </svg>
                </IconButton>
            )}
        </div>
    )
}
