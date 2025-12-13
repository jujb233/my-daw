import { render } from 'solid-js/web'

function App() {
        return (
                <div>
                        <h2>Simple Synth UI</h2>
                        <p>
                                Frequency: <span id='freq'>440</span>
                        </p>
                        <input id='freq-slider' type='range' min='20' max='2000' defaultValue='440' />
                </div>
        )
}

render(() => <App />, document.getElementById('root')!)
