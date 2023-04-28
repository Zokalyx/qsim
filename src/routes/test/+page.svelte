<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri'

    enum PotentialMode {
        Drawing,
        Formula,   
    }
    interface Potential {
        mode: PotentialMode,
        formula: string,
        sketching: boolean,
        formula_error: string,
        datapoints: Datapoint[] | null,
    }
    interface Bounds {
        position: {
            min: number,
            max: number,
        },
        amplitude: {
            min: number,
            max: number,
        },
    }
    // Wrapper required for serde serialization/deserialization
    interface Datapoints {
        values: Datapoint[],
    }
    interface Datapoint {
        x: number,
        y: number,
    }

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
    let potential: Potential = {
        mode: PotentialMode.Formula,
        sketching: false,
        formula: "x^2",
        formula_error: "",
        datapoints: null,
    }

    function path(datapoints: Datapoint[]) {
        console.log(datapoints)
        return "M " + datapoints.map((datapoint) => `${datapoint.x} ${datapoint.y}`).join(" ")
    }

    async function validate_formula() {
        potential.formula_error = await invoke("formula_error", { formula: potential.formula })
    }

    async function compute_formula() {
        potential.datapoints = (await invoke(
            "compute_formula", { 
                formula: potential.formula,
                start: bounds.position.min,
                end: bounds.position.max,
                resolution: 100,
            }) as Datapoints).values
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

<h1>
    Potential
</h1>

<select bind:value={potential.mode}>
    <option value={PotentialMode.Formula}>
        Formula
    </option>
    <option value={PotentialMode.Drawing}>
        Drawing
    </option>
</select>

{#if potential.mode === PotentialMode.Formula}
<input bind:value={potential.formula} on:input={validate_formula}>
    {#if !potential.formula_error}
        <button on:click={compute_formula}>
            Update
        </button>
    {:else}
        <span>
            {potential.formula_error}
        </span>
    {/if}
{:else if potential.mode === PotentialMode.Drawing}
<button>
    {!potential.sketching ? "Start drawing" : "Stop drawing"}
</button>
{/if}

<h1>
    Graph
</h1>
<svg height="200" width="500">
    {#if potential.datapoints}
    <path fill="none" stroke="red" d={path(potential.datapoints)}/>
    {/if}
</svg>