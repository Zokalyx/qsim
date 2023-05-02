<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri'
    import { type Function, Datapoints, type Bounds } from '$lib/types'
    import { FunctionMode } from '$lib/types'
    import type { Writable } from 'svelte/store'
	import { onMount } from 'svelte';
    import alert from "$lib/static/alert-circle-svgrepo-com.svg"
    import help from "$lib/static/help-circle-svgrepo-com.svg"
    import Tooltip from "$lib/static/Tooltip.svelte";
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    onMount(() => {
        compute_formula()
    })

    export let N: number
    export let bounds: Bounds
    export let fn: Writable<Function>
    export let currently_drawing: Writable<Function | null>
    export let playing: Writable<boolean>

    function dispatch_update() {
        dispatch("update")
    }

    async function compute_formula() {
        if ($fn.readonly) {
            return
        }

        $currently_drawing = null
        dispatch("update")
        $fn.formula_error = await invoke("formula_error", { formula: $fn.formula })
        if (!$fn.formula_error) {
            $fn.datapoints = new Datapoints((await invoke(
                "compute_formula", { 
                    formula: $fn.formula,
                    start: bounds.left,
                    end: bounds.right,
                    resolution: N,
                    normalize: $fn.name === "Wavefunction",
                }) as Datapoints).values)
        }
    }

    function start_drawing() {
        $currently_drawing = $fn
    }

    function stop_drawing() {
        $currently_drawing = null
    }

    $: bounds ? compute_formula() : undefined
    $: N ? compute_formula() : undefined
    // $: !$playing ? compute_formula() : undefined
</script>

<div class="border-2 border-black rounded-lg bg-zinc-700 ml-2 mt-2 p-3 flex flex-col space-y-3">
    <div class="flex space-x-3 items-center">
        <h1 class="text-3xl text-slate-100 font-bold">
            {$fn.name}
        </h1>
        <input type="checkbox" bind:checked={$fn.visible} class="outline-3 outline-black w-4 h-4"/>
    </div>

    {#if !$fn.readonly}
    <div class="flex space-x-3 items-center">
        <select bind:value={$fn.mode} on:change={() => $fn.mode === FunctionMode.Formula ? compute_formula() : null} class="border-2 border-black h-10 rounded-md px-2 py-1 focus:outline-none focus:bg-slate-50 bg-slate-300 transition-colors duration-300 text-slate-900">
            <option value={FunctionMode.Formula}>
                Formula
            </option>
            <option value={FunctionMode.Drawing}>
                Drawing
            </option>
        </select>

        {#if $fn.mode === FunctionMode.Formula}
        <input bind:value={$fn.formula} on:input={compute_formula} class="border-2 border-black h-10 rounded-md px-2 py-1 focus:outline-none focus:bg-slate-50 bg-slate-300 transition-colors duration-300 text-slate-900">
        {#if $fn.formula_error}
        <Tooltip title={$fn.formula_error}>
            <img alt="error" src="{alert}" class="w-7 h-7"/>
        </Tooltip>
        {/if}
        {:else if $fn.mode === FunctionMode.Drawing}
            {#if $fn !== $currently_drawing}
            <button on:click={start_drawing} class="border-2 border-black rounded-md p-2 text-slate-900 bg-slate-300 focus:bg-slate-50 transition-colors duration-300">
                Draw
            </button>
            {:else}
            <button on:click={stop_drawing} class="border-2 border-black rounded-md p-2 text-slate-900 bg-slate-300 focus:bg-slate-50 transition-colors duration-300">
                Stop
            </button>
            {/if}
        {/if}
    </div>
    {/if}

    {#if $fn.n !== null}
    <div class="flex space-x-3 items-center">
        <span>Quantum number: </span>
        <input type="range" bind:value={$fn.n} on:input={dispatch_update} min="0" max="{N/2}"/>
        <span>{$fn.n}</span>
    </div>
    {/if}

    <div class="flex space-x-3 items-center">
        <span>Scale:</span>
        <input type="number" bind:value={$fn.scale.bottom} class="border-2 w-20 border-black h-10 rounded-md px-2 py-1 focus:outline-none focus:bg-slate-50 bg-slate-300 transition-colors duration-300 text-slate-900">
        <span>-</span>
        <input type="number" bind:value={$fn.scale.top} class="border-2 w-20 border-black h-10 rounded-md px-2 py-1 focus:outline-none focus:bg-slate-50 bg-slate-300 transition-colors duration-300 text-slate-900">
    </div>

    {#if $fn.complex_phase !== null}
    <div class="flex space-x-3 items-center">
        <span>Phase factor:</span>
        <input min="-10" max="10" type="number" bind:value={$fn.complex_phase} on:input={() => {dispatch_update(); compute_formula();}} class="border-2 w-20 border-black h-10 rounded-md px-2 py-1 focus:outline-none focus:bg-slate-50 bg-slate-300 transition-colors duration-300 text-slate-900">
        <Tooltip title="Complex phase factor associated with momentum">
            <img alt="help" src="{help}" class="w-7 h-7"/>
        </Tooltip>
    </div>
    {/if}
</div>
