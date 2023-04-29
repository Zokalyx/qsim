<script lang="ts">
	import type { Bounds, Datapoints, Function } from "$lib/types";
    import type { Writable } from 'svelte/store'

    // Todo: make responsive
    const WIDTH = 700
    const HEIGHT = 300

    export let bounds: Bounds
    export let potential: Writable<Function>
    export let wavefunction: Writable<Function>
</script>

<h1>
    Graph
</h1>
<svg height={HEIGHT} width={WIDTH}>
    {#if $potential.datapoints}
    <path fill="none" stroke="red" d={$potential.datapoints.get_path(WIDTH, HEIGHT, bounds)}/>
    {/if}

    {#if $wavefunction.datapoints}
    <path fill="none" stroke="green" d={$wavefunction.datapoints.get_path(WIDTH, HEIGHT, bounds)}/>
    {/if}
</svg>