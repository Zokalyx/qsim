<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri'
    import { type Function, Datapoints, type Bounds } from '$lib/types'
    import { FunctionMode } from '$lib/types'
    import type { Writable } from 'svelte/store'

    export let bounds: Bounds
    export let fn: Writable<Function>

    async function validate_formula() {
        $fn.formula_error = await invoke("formula_error", { formula: $fn.formula })
    }

    async function compute_formula() {
        $fn.datapoints = new Datapoints((await invoke(
            "compute_formula", { 
                formula: $fn.formula,
                start: bounds.position.min,
                end: bounds.position.max,
                resolution: 100,
            }) as Datapoints).values)
    }
</script>

<h1>
    {$fn.name}
</h1>

<select bind:value={$fn.mode}>
    <option value={FunctionMode.Formula}>
        Formula
    </option>
    <option value={FunctionMode.Drawing}>
        Drawing
    </option>
</select>

{#if $fn.mode === FunctionMode.Formula}
<input bind:value={$fn.formula} on:input={validate_formula}>
    {#if !$fn.formula_error}
        <button on:click={compute_formula}>
            Update
        </button>
    {:else}
        <span>
            {$fn.formula_error}
        </span>
    {/if}
{:else if $fn.mode === FunctionMode.Drawing}
<button>
    {!$fn.sketching ? "Start drawing" : "Stop drawing"}
</button>
{/if}
