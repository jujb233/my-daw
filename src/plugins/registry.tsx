import { Component } from 'solid-js'
import { SimpleSynth } from './SimpleSynth'

export const PluginUIRegistry: Record<string, Component<any>> = {
    'com.mydaw.simplesynth': SimpleSynth
}
