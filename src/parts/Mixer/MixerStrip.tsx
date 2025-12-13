import { Component } from 'solid-js'
import { MixerTrackData, setTrackVolume, removeMixerTrack, toggleMute, toggleSolo } from '../../store/mixer'
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
                <div class='bg-surface-container-low border-outline-variant flex w-28 shrink-0 flex-col items-center gap-3 border-r py-3 select-none'>
                        {/* Header / Label */}
                        <div class='w-full px-2 text-center'>
                                <span
                                        class='text-on-surface block truncate text-sm font-medium'
                                        title={props.track.label}
                                >
                                        {props.track.label}
                                </span>
                        </div>

                        {/* Pan / Other controls placeholder */}
                        {/* Made larger for touch */}
                        <div class='border-outline-variant bg-surface hover:bg-surface-container-high flex h-10 w-10 cursor-pointer items-center justify-center rounded-full border'>
                                <div class='bg-primary h-4 w-0.5 rotate-0' />
                        </div>

                        {/* Meter & Fader Container */}
                        <div class='bg-surface-container-lowest mx-1 flex w-full flex-1 items-end justify-center gap-3 rounded-lg px-2 py-2 shadow-inner'>
                                <LevelMeter level={props.level} className='h-64 w-11' />
                                <Fader
                                        value={props.track.volume}
                                        onChange={v => setTrackVolume(props.track.id, v)}
                                        className='h-64 w-12'
                                        defaultValue={0.75}
                                />
                        </div>

                        {/* Mute / Solo / Rec */}
                        <div class='flex w-full justify-center gap-2 px-2'>
                                <button
                                        title={t('icons.mute')}
                                        class={`h-10 flex-1 rounded text-xs font-bold transition-colors ${
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
                                        class={`h-10 flex-1 rounded text-xs font-bold transition-colors ${
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
                                        class='text-error hover:bg-error/10 h-10 w-10'
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
