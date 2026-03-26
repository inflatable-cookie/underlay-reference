<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import "@poodle/svelte-tokens/styles.css";
	import "@poodle/svelte-tokens/theme-light.css";
	import "@poodle/svelte-tokens/density-comfortable.css";
	import "@poodle/svelte-tokens/control-size-md.css";
	import "@decodelabs/underlay/styles/base.css";
	import "@decodelabs/underlay/styles/tokens.css";
	import "@decodelabs/underlay/styles/forms.css";

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div
	class="container front-theme-shell"
	data-theme="light"
	data-density="comfortable"
	data-control-size="md"
>
	{@render children()}
</div>

<style>
	:global(body) {
		margin: 0;
		font-family:
			"Avenir Next",
			Avenir,
			system-ui,
			-ui-sans-serif,
			-apple-system,
			BlinkMacSystemFont,
			"Segoe UI",
			sans-serif;
		background: var(--poodle-color-background-canvas);
		color: var(--poodle-color-text-primary);
	}

	:global(a) {
		color: inherit;
		text-decoration: underline;
		text-underline-offset: 0.15em;
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
