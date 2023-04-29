<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri'
    import type { Function, Datapoint, Datapoints, Bounds } from '$lib/types'
    import { FunctionMode } from "$lib/types"
    import Formula from '$lib/Formula.svelte'
    import Graph from '$lib/Graph.svelte'
    import { writable } from 'svelte/store'

    let bounds: Bounds = {
        position: {
            min: -10,
            max: 10,
        },
        amplitude: {
            min: -10,
            max: 10,
        },
    }
    let potential = writable({
        name: "Potential",
        mode: FunctionMode.Formula,
        sketching: false,
        formula: "x^2",
        formula_error: "",
        datapoints: null,
    })
    let wavefunction = writable({
        name: "Wavefunction",
        mode: FunctionMode.Formula,
        sketching: false,
        formula: "2.787^(-x^2)",
        formula_error: "",
        datapoints: null,
    })

    async function simulate() {
        await invoke("simulate", { 
            potential: $potential.formula,
            wavefunction: $wavefunction.formula,
            start: bounds.position.min,
            end: bounds.position.max,
            resolution: 100,
        })
    }
</script>

<h1>
    Position bounds
</h1>
<input type="number" bind:value={bounds.position.min}>
to
<input type="number" bind:value={bounds.position.max}>
<h1>
    Amplitude bounds
</h1>
<input type="number" bind:value={bounds.amplitude.min}>
to
<input type="number" bind:value={bounds.amplitude.max}>

<Formula fn={potential} bounds={bounds} />
<Formula fn={wavefunction} bounds={bounds} />

<Graph potential={potential} wavefunction={wavefunction} bounds={bounds} />

<button on:click={simulate}>
    Simulate
</button>