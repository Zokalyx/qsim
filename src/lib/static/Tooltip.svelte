<script lang="ts">
    //https://svelte.dev/repl/dd6754a2ad0547c5b1c1ea37c0293fef?version=3.58.0
	export let title = '';
	let isHovered = false;
	let x: number;
	let y: number;
	
	function mouseOver(event: MouseEvent) {
		isHovered = true;
		x = event.pageX + 5;
		y = event.pageY + 5;
	}
	function mouseMove(event: MouseEvent) {
		x = event.pageX + 5;
		y = event.pageY + 5;
	}
	function mouseLeave() {
		isHovered = false;
	}
</script>

<div
	on:mouseover={mouseOver}
    on:mouseleave={mouseLeave}
	on:mousemove={mouseMove}>
	<slot />
</div>

{#if isHovered}
	<div style="top: {y}px; left: {x}px;" class="tooltip absolute p-1 shadow-sm shadow-zinc-800 rounded-md border-2 border-black bg-zinc-200 text-zinc-800">{title}</div>
{/if}
