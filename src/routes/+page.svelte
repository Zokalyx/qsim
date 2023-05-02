<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
	import type { Function, Datapoint, Bounds } from '$lib/types';
    import { Datapoints } from "$lib/types";
	import { FunctionMode } from '$lib/types';
	import Formula from '$lib/Formula.svelte';
	import Graph from '$lib/Graph.svelte';
	import { writable, type Writable } from 'svelte/store';
    import { onMount } from 'svelte';
	import alert from "$lib/static/alert-circle-svgrepo-com.svg";
	import Tooltip from '../lib/static/Tooltip.svelte';

    let N = 100
	let max_N = 200
	let currently_drawing: Writable<Function | null> = writable(null)
	let playing = writable(false)
	let time = writable(0)
	let parametersChanged = writable(true)

	let bounds: Bounds = {
		left: -10,
		right: 10,
	};
	let potential: Writable<Function> = writable({
		name: 'Potential',
		mode: FunctionMode.Formula,
		sketching: false,
		formula: '0.1 * x^2',
		formula_error: '',
		datapoints: null,
		show_mean: false,
		scale: {
			top: 2,
			bottom: -2,
		},
		visible: true,
		complex_phase: null,
		readonly: false,
		n: null,
	});
	let wavefunction: Writable<Function> = writable({
		name: 'Wavefunction',
		mode: FunctionMode.Formula,
		sketching: false,
		formula: 'e^(-x^2)',
		formula_error: '',
		datapoints: null,
		show_mean: true,
		scale: {
			top: 1.5,
			bottom: -1.5
		},
		visible: true,
		complex_phase: 3,
		readonly: false,
		n: null
	});
	let eigenfunction: Writable<Function> = writable({
		name: 'Eigenfunction',
		mode: FunctionMode.Formula,
		sketching: false,
		formula: '',
		formula_error: '',
		datapoints: null,
		show_mean: false,
		scale: {
			top: 1,
			bottom: -1,
		},
		visible: true,
		complex_phase: null,
		readonly: true,
		n: 0,
	});

	async function interrupt() {
		$parametersChanged = true
		$time = 0
		$playing = false
		$eigenfunction.datapoints = null
		await invoke("restart")
	}

	async function get_eigenvector() {
		$eigenfunction.datapoints = new Datapoints((await invoke(
            "get_eigenvector", { 
                n: $eigenfunction.n,
                start: bounds.left,
                end: bounds.right,
                resolution: N,
            }) as Datapoints).values)
	}
</script>

<div class="flex flex-col py-10 items-center">
<div class="flex justify-stretch">
	<div class="border-2 border-black rounded-lg bg-zinc-700 ml-2 mt-2 p-3 flex flex-col space-y-3">
		<h1 class="text-3xl text-slate-100 font-bold">Bounds</h1>
		<div>
			<input type="number" bind:value={bounds.left} on:input={interrupt} class="border-2 w-20 border-black h-10 rounded-md px-2 py-1 focus:outline-none focus:bg-slate-50 bg-slate-300 transition-colors duration-300 text-slate-900"/>
			-
			<input type="number" bind:value={bounds.right} on:input={interrupt} class="border-2 w-20 border-black h-10 rounded-md px-2 py-1 focus:outline-none focus:bg-slate-50 bg-slate-300 transition-colors duration-300 text-slate-900"/>
			<!--h1>Amplitude bounds</h1>
			<input type="number" bind:value={bounds.amplitude.min} />
			to
			<input type="number" bind:value={bounds.amplitude.max} /-->
		</div>
		<div class="flex items-center space-x-3">
			<span>Divisions:</span>
			<input type="number" bind:value={N} class="border-2 w-20 border-black h-10 rounded-md px-2 py-1 focus:outline-none focus:bg-slate-50 bg-slate-300 transition-colors duration-300 text-slate-900"/>
			{#if N >= max_N}
			<Tooltip title="Values above ~{max_N} may lag the program">
				<img alt="error" src="{alert}" class="w-7 h-7"/>
			</Tooltip>
			{/if}
		</div>
	</div>

	<Formula fn={potential} {bounds} N={N} on:update={interrupt} {currently_drawing} {playing}/>
	<Formula fn={wavefunction} {bounds} N={N} on:update={interrupt} currently_drawing={currently_drawing} {playing}/>
	<Formula fn={eigenfunction} {bounds} N={N} {currently_drawing} {playing} on:update={get_eigenvector}/>
</div>

<Graph {potential} {wavefunction} {eigenfunction} {bounds} {currently_drawing} {N} {playing} {time} on:eigenpls={get_eigenvector} on:interrupt={interrupt} {parametersChanged}/>
</div>