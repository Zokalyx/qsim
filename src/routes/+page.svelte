<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
	import type { Function, Datapoint, Bounds } from '$lib/types';
    import { Datapoints } from "$lib/types";
	import { FunctionMode } from '$lib/types';
	import Formula from '$lib/Formula.svelte';
	import Graph from '$lib/Graph.svelte';
	import { writable, type Writable } from 'svelte/store';
    import { onMount } from 'svelte';

    let N = 100
    let time = 0
    let playing = false
    let speed = 0.10
	let momentum = 0

    onMount(() => {
        const interval = setInterval(() => {
            if (playing) {
                time += speed
                evolve()
            }
        }, 1000 / 75)

        return () => clearInterval(interval)
    })

	let bounds: Bounds = {
		position: {
			min: -10,
			max: 10
		},
		amplitude: {
			min: -2,
			max: 2
		}
	};
	let potential = writable({
		name: 'Potential',
		mode: FunctionMode.Formula,
		sketching: false,
		formula: '0.1 * x^2',
		formula_error: '',
		datapoints: null,
		show_mean: false,
	});
	let wavefunction: Writable<Function> = writable({
		name: 'Wavefunction',
		mode: FunctionMode.Formula,
		sketching: false,
		formula: '2.787^(-x^2)',
		formula_error: '',
		datapoints: null,
		show_mean: true,
	});
	let eigenfunction: Writable<Function> = writable({
		name: 'Eigenfunction',
		mode: FunctionMode.Formula,
		sketching: false,
		formula: '',
		formula_error: '',
		datapoints: null,
		show_mean: false,
	});

    let n = 0;

    function toggle_play() {
        playing = !playing
    }

	async function simulate() {
        time = 0

		await invoke('simulate', {
			potential: $potential.formula,
			wavefunction: $wavefunction.formula,
			start: bounds.position.min,
			end: bounds.position.max,
			resolution: N,
			momentum: momentum,
		});

        await get_eigenvector()
	}

	async function get_eigenvector() {
		$eigenfunction.datapoints = new Datapoints((await invoke(
            "get_eigenvector", { 
                n,
                start: bounds.position.min,
                end: bounds.position.max,
                resolution: N,
            }) as Datapoints).values)
	}

    async function evolve() {
        $wavefunction.datapoints = new Datapoints((await invoke(
            "evolve", {
                time,
                start: bounds.position.min,
                end: bounds.position.max,
            }) as Datapoints).values)
    }
</script>

<h1>Position bounds</h1>
<input type="number" bind:value={bounds.position.min} />
to
<input type="number" bind:value={bounds.position.max} />
<h1>Amplitude bounds</h1>
<input type="number" bind:value={bounds.amplitude.min} />
to
<input type="number" bind:value={bounds.amplitude.max} />

<Formula fn={potential} {bounds} N={N} />
<Formula fn={wavefunction} {bounds} N={N}/>

<Graph {potential} {wavefunction} {eigenfunction} {bounds} />

<br>

<button on:click={simulate}> Simulate </button>

Eigenvector
<input type="range" min="0" max="{N/2}" bind:value={n} on:input={get_eigenvector}>
{n}

<input type="number" bind:value={N} />

<br>

Initial Momentum
<input type="number" min="0" max="{10}" bind:value={momentum}>
{momentum}

<br>

<button on:click={toggle_play}>
    {#if playing}
    Pause
    {:else}
    Play
    {/if}
</button>

Speed
<input type="range" min="0.0" max="1.0" step="0.01" bind:value={speed}>
{speed}