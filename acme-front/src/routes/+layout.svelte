<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import "@inflatable-cookie/poodle-svelte-tokens/styles.css";
	import "@inflatable-cookie/poodle-svelte-tokens/theme-iceberg.css";
	import "@inflatable-cookie/poodle-svelte-tokens/density-comfortable.css";
	import "@inflatable-cookie/poodle-svelte-tokens/control-size-md.css";
	import "@inflatable-cookie/underlay/styles/base.css";
	import "@inflatable-cookie/underlay/styles/tokens.css";
	import "@inflatable-cookie/underlay/styles/forms.css";

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div
	class="container front-theme-shell"
	data-theme="iceberg"
	data-density="comfortable"
	data-control-size="md"
>
	{@render children()}
</div>

<style>
	:global(body) {
		margin: 0;
		font-family: var(--poodle-typography-font-family-sans);
		background: var(--poodle-color-background-canvas);
		color: var(--poodle-color-text-primary);
		--poodle-surface: var(--poodle-color-background-canvas);
	}

	.container {
		max-width: 980px;
		margin: 0 auto;
		padding: 1.25rem;
	}

	:global(.front-theme-shell) {
		min-height: 100vh;
	}
</style>
