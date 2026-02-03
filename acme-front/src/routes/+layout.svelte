<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import "@decodelabs/underlay/styles/base.css";
	import "@decodelabs/underlay/styles/tokens.css";
	import "@decodelabs/underlay/styles/forms.css";

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="container">
	{@render children()}
</div>

<style>
	:global(body) {
		margin: 0;
		font-family: ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial,
			sans-serif;
		background: radial-gradient(1200px 600px at 50% -10%, rgba(240, 253, 250, 0.85), transparent),
			linear-gradient(180deg, #f8fafc, #ffffff);
		color: #0f172a;
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
</style>
