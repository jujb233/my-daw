import { Component, createSignal } from 'solid-js'
import { open } from '@tauri-apps/plugin-dialog'
import { DawService } from '../services/daw'

interface Props {
        onDone: () => void
}

export const ProjectChooser: Component<Props> = props => {
        const [visible, setVisible] = createSignal(true)
        const [loading, setLoading] = createSignal(false)

        const handleLoad = async () => {
                try {
                        const selected = await open({ directory: true, multiple: false, title: 'Load Project' })
                        if (selected && typeof selected === 'string') {
                                setLoading(true)
                                await DawService.loadProject(selected)
                                // ensure plugins in project are scanned
                                await DawService.scanProjectPlugins(selected)
                                setLoading(false)
                                setVisible(false)
                                props.onDone()
                        }
                } catch (e) {
                        console.error('Failed to load project', e)
                        setLoading(false)
                }
        }

        const handleCreate = async () => {
                try {
                        const selected = await open({
                                directory: true,
                                multiple: false,
                                title: 'Select Folder For New Project'
                        })
                        if (selected && typeof selected === 'string') {
                                setLoading(true)
                                // Create initial project by saving current (in-memory) state to this folder
                                await DawService.saveProject(selected)
                                // Load it back
                                await DawService.loadProject(selected)
                                // scan project plugins
                                await DawService.scanProjectPlugins(selected)
                                setLoading(false)
                                setVisible(false)
                                props.onDone()
                        }
                } catch (e) {
                        console.error('Failed to create project', e)
                        setLoading(false)
                }
        }

        if (!visible()) return null

        return (
                <div class='fixed inset-0 z-50 flex items-center justify-center bg-black/40'>
                        <div class='bg-surface w-[520px] rounded-lg p-6'>
                                <h3 class='mb-4 text-lg font-semibold'>Open or Create Project</h3>
                                <p class='text-on-surface-variant mb-6 text-sm'>
                                        Choose to open an existing project folder or create a new project at a chosen
                                        location.
                                </p>
                                <div class='flex gap-3'>
                                        <button
                                                class='bg-primary rounded px-4 py-2 text-white'
                                                onClick={handleLoad}
                                                disabled={loading()}
                                        >
                                                {loading() ? 'Loading...' : 'Load Existing Project'}
                                        </button>
                                        <button
                                                class='rounded border px-4 py-2'
                                                onClick={handleCreate}
                                                disabled={loading()}
                                        >
                                                {loading() ? 'Creating...' : 'Create New Project'}
                                        </button>
                                        <button
                                                class='rounded px-4 py-2'
                                                onClick={() => {
                                                        setVisible(false)
                                                        props.onDone()
                                                }}
                                        >
                                                Cancel
                                        </button>
                                </div>
                        </div>
                </div>
        )
}
