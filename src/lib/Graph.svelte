<script lang="ts">
	import type { Bounds, Function } from "$lib/types";
    import type { Writable } from 'svelte/store'

    // Todo: make responsive
    const WIDTH = 700
    const HEIGHT = 300

    export let bounds: Bounds
    export let potential: Writable<Function>
    export let wavefunction: Writable<Function>
    export let eigenfunction: Writable<Function>

    $: zero_height = bounds.amplitude.max * HEIGHT / (bounds.amplitude.max - bounds.amplitude.min)
    $: wavefunction_mean = $wavefunction.datapoints ? ($wavefunction.datapoints.get_mean() - bounds.position.min) * WIDTH / (bounds.position.max - bounds.position.min) : 0
</script>

<h1>
    Graph
</h1>
<svg height={HEIGHT} width={WIDTH}>
    <path fill="none" stroke="black" d={`M ${0} ${zero_height} ${WIDTH} ${zero_height}`}/>

    {#if $potential.datapoints}
    <path fill="none" stroke="red" d={$potential.datapoints.get_path(WIDTH, HEIGHT, bounds)}/>
    {/if}

    {#if $wavefunction.datapoints}
    <path fill="none" stroke="green" d={$wavefunction.datapoints.get_path(WIDTH, HEIGHT, bounds)}/>
    <path fill="none" stroke="green" d={`M ${wavefunction_mean} ${0} ${wavefunction_mean} ${HEIGHT}`}/>
    {/if}

    {#if $eigenfunction.datapoints}
    <path fill="none" stroke="orange" d={$eigenfunction.datapoints.get_path(WIDTH, HEIGHT, bounds)}/>
    {/if}
</svg>