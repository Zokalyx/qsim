<script lang="ts">
    export let href: string | null = null
</script>

{#if href}
<a href={href}>
<button>
    <slot />
</button>
</a>
{:else}
<button>
    <slot />
</button>
{/if}