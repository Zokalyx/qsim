<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri'

    // Todo: make responsive
    const WIDTH = 700
    const HEIGHT = 300

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

    function fit_to_screen(datapoints: Datapoint[]) {
        // maps selected bounds to screen size
        let left = bounds.position.min  // -> 0
        let right = bounds.position.max  // -> WIDTH
        // "Slope" of Width / (right - left)
        // "Root" at left
        // Result: (x - left) * Width / (right - left)

        let bottom = bounds.amplitude.min  // -> HEIGHT
        let top = bounds.amplitude.max  // --> 0
        // "Slope" of -Height / (right - left)
        // "Root" at top
        // Result: - (x - top) * Height / (right - left)
        
        datapoints.forEach(datapoint => {
            datapoint.x = (datapoint.x - left) * WIDTH / (right - left)
            datapoint.y = -(datapoint.y - top) * HEIGHT / (right - left)
        })

        return datapoints
    }

    async function validate_formula() {
        potential.formula_error = await invoke("formula_error", { formula: potential.formula })
    }

    async function compute_formula() {
        let raw_data = (await invoke(
            "compute_formula", { 
                formula: potential.formula,
                start: bounds.position.min,
                end: bounds.position.max,
                resolution: 100,
            }) as Datapoints).values
        potential.datapoints = fit_to_screen(raw_data)
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
<svg height="{HEIGHT}" width="{WIDTH}">
    {#if potential.datapoints}
    <path fill="none" stroke="red" d={path(potential.datapoints)}/>
    {/if}
</svg>