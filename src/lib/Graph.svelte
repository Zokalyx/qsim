<script lang="ts">
	import type { Bounds, Datapoint, Function } from "$lib/types";
    import type { Writable } from 'svelte/store'
    import { createEventDispatcher, onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/tauri";
    import { Datapoints } from "$lib/types";
    import { FunctionMode } from "$lib/types";

    // Todo: make responsive
    const WIDTH = 700
    const HEIGHT = 300

    export let bounds: Bounds
    export let potential: Writable<Function>
    export let wavefunction: Writable<Function>
    export let eigenfunction: Writable<Function>
    export let currently_drawing: Writable<Function | null>
    export let N: number
    export let playing: Writable<boolean>
    export let time: Writable<number>
    export let parametersChanged: Writable<boolean>
    let mousedown = false
    let svg: Element
    let helper = 1
    let speed = 0.10

    $: error_free = (!$wavefunction.formula_error || $wavefunction.mode === FunctionMode.Drawing) && (!$potential.formula_error || $potential.mode === FunctionMode.Drawing)

    const dispatch = createEventDispatcher()

    onMount(() => {
        const interval = setInterval(() => {
            if ($playing) {
                $time += speed
                evolve()
            }
        }, 1000 / 75)

        return () => clearInterval(interval)
    })

    async function evolve() {
        $wavefunction.datapoints = new Datapoints((await invoke(
            "evolve", {
                time: $time,
                start: bounds.left,
                end: bounds.right,
            }) as Datapoints).values)
    }

    function handleMousedown(e: MouseEvent) {
        mousedown = true
        try_draw(e)
    }

    function handleMouseup() {
        mousedown = false
    }

    function reset() {
        $time = 0
        evolve()
    }

    function toggle_play() {
        $playing = !$playing
    }

	async function simulate() {
        if (!$parametersChanged) {
            $playing = true
            return
        }

        $currently_drawing = null
        $time = 0
		$playing = true
        $parametersChanged = false

		await invoke('simulate', {
			potentialFormula: $potential.formula,
            potentialDatapoints: $potential.datapoints,
            usePotentialFormula: $potential.mode === FunctionMode.Formula,
			wavefunctionFormula: $wavefunction.formula,
            wavefunctionDatapoints: $wavefunction.datapoints,
            useWavefunctionFormula: $wavefunction.mode === FunctionMode.Formula,
			start: bounds.left,
			end: bounds.right,
			resolution: N,
			momentum: $wavefunction.complex_phase,
		});

        dispatch("eigenpls")
	}

    function try_draw(e: MouseEvent) {
        if (!$currently_drawing || !mousedown) {
            return
        }
        dispatch("interrupt")
        let rect = svg.getBoundingClientRect()
        let raw_x = e.clientX - rect.left
        let raw_y = e.clientY - rect.top

        // raw_x = 0 --> bound.left
        // raw_x = rect.width --> bound.right
        let x = bounds.left + (bounds.right - bounds.left) * raw_x / rect.width

        // raw_y = 0 --> scale.top
        // raw_y = rect.height --> scale.bottom
        let y = $currently_drawing.scale.top - raw_y / rect.height * ($currently_drawing.scale.top - $currently_drawing.scale.bottom)

        // Find nearest datapoint and update it.
        if (!$currently_drawing.datapoints) {
            return
        }
        let min = null
        let index = null
        for (const [i, d] of $currently_drawing.datapoints.values.entries()) {
            let diff = Math.abs(d.x - x)

            if (!min) {
                min = diff
                index = i
            }

            if (diff < min) {
                min = diff
                index = i
            }
        }
        if (!index) {
            return
        }

        helper += 1
        $currently_drawing.datapoints.values[index].y = y
    }

    // $: zero_height = bounds.amplitude.max * HEIGHT / (bounds.amplitude.max - bounds.amplitude.min)
    $: wavefunction_mean = $wavefunction.datapoints ? ($wavefunction.datapoints.get_mean() - bounds.left) * WIDTH / (bounds.right - bounds.left) : 0
</script>

<div class="border-2 border-black rounded-lg bg-zinc-700 ml-2 mt-2 p-3" on:mouseleave|self={handleMouseup} on:mousedown={handleMousedown} on:mouseup={handleMouseup} >
<!--h1 class="text-3xl text-zinc-100 font-bold">
    Graph
</h1-->
<div class="flex space-x-2">
    {#key helper}
    <svg bind:this={svg} on:mousemove={try_draw} height={HEIGHT} width={WIDTH} class="border-2 bg-zinc-300 rounded-md border-black">
        <!--path fill="none" stroke="black" d={`M ${0} ${zero_height} ${WIDTH} ${zero_height}`}/-->

        {#if $potential.datapoints && $potential.visible}
        <path stroke-width="3" fill="none" stroke="red" d={$potential.datapoints.get_path(WIDTH, HEIGHT, bounds, $potential.scale)}/>
        {/if}

        {#if $wavefunction.datapoints && $wavefunction.visible}
        <path stroke-width="3" fill="none" stroke="green" d={$wavefunction.datapoints.get_path(WIDTH, HEIGHT, bounds, $wavefunction.scale)}/>
        <!--path stroke-width="3" fill="none" stroke="green" d={`M ${wavefunction_mean} ${0} ${wavefunction_mean} ${HEIGHT}`}/-->
        {/if}

        {#if $eigenfunction.datapoints && $eigenfunction.visible}
        <path stroke-width="3" fill="none" stroke="orange" d={$eigenfunction.datapoints.get_path(WIDTH, HEIGHT, bounds, $eigenfunction.scale)}/>
        {/if}
    </svg>
    {/key}

    <div class="flex flex-col space-y-3">
        <div class="flex space-x-3 items-center">
        <h1 class="text-3xl text-zinc-100 font-bold">
        Controles
        </h1>
        </div>

        <div class="flex space-x-3 items-center">
            {#if !$playing}
            <button disabled="{!error_free}" on:click={simulate} class="border-2 border-black rounded-md p-2 text-zinc-900 bg-zinc-300 focus:bg-zinc-50 transition-colors duration-300 disabled:text-zinc-500">
            Simular
            </button>
            {:else}
            <button on:click={toggle_play} class="border-2 border-black rounded-md p-2 text-zinc-900 bg-zinc-300 focus:bg-zinc-50 transition-colors duration-300 disabled:text-zinc-500">
            Pausar
            </button>
            <button on:click={reset} class="border-2 border-black rounded-md p-2 text-zinc-900 bg-zinc-300 focus:bg-zinc-50 transition-colors duration-300">
            Reiniciar
            </button>
            {/if}
        </div>

        <div class="flex space-x-3 items-center">
        <span>Velocidad</span>
        <input type="range" min="0.01" max="1.0" step="0.01" bind:value={speed}>
        </div>
        </div>
    </div>
</div>