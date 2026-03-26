<script lang="ts">
	import { page } from '$app/stores';
	import favicon from '$lib/assets/favicon.svg';
	import { resolveAdminPageTitle } from '$lib/utils/page-title';
	import "@poodle/svelte-tokens/styles.css";
	import "@poodle/svelte-tokens/theme-dark.css";
	import "@poodle/svelte-tokens/density-compact.css";
	import "@poodle/svelte-tokens/control-size-md.css";
	import "@decodelabs/underlay/styles/base.css";
	// Import Underlay CSS FIRST so our :root overrides take precedence.
	import "@decodelabs/underlay/styles/tokens.css";
	import "@decodelabs/underlay/styles/forms.css";

	let { children } = $props();
</script>

<svelte:head>
	<title>{resolveAdminPageTitle($page.route.id, $page.url.pathname)}</title>
	<link rel="icon" href={favicon} />
</svelte:head>

<div
	class="admin-theme-shell"
	data-theme="dark"
	data-density="compact"
	data-control-size="md"
>
	{@render children()}
</div>

<style>
	:global(.admin-theme-shell) {
		/* Core colors */
		--admin-color-bg: var(--poodle-color-background-canvas);
		--admin-color-surface: color-mix(in srgb, var(--poodle-color-background-panel) 72%, #050608);
		--admin-color-surface-subtle: color-mix(in srgb, var(--poodle-color-background-panel) 88%, #16181d);
		--admin-color-surface-card: color-mix(in srgb, var(--poodle-color-background-surface) 82%, #101318);
		--admin-color-border-subtle: color-mix(in srgb, var(--poodle-color-border-subtle) 72%, transparent);
		--admin-color-border-strong: color-mix(in srgb, var(--poodle-color-border-default) 84%, transparent);
		--admin-color-text: var(--poodle-color-text-primary);
		--admin-color-text-muted: var(--poodle-color-text-secondary);
		--admin-color-accent: var(--poodle-color-accent-base);

		/* Status colors */
		--admin-color-success: var(--poodle-color-status-success);
		--admin-color-success-subtle: color-mix(in srgb, var(--poodle-color-status-success) 18%, transparent);
		--admin-color-warning: var(--poodle-color-status-warning);
		--admin-color-warning-subtle: color-mix(in srgb, var(--poodle-color-status-warning) 18%, transparent);
		--admin-color-danger: var(--poodle-color-status-danger);
		--admin-color-danger-subtle: color-mix(in srgb, var(--poodle-color-status-danger) 18%, transparent);
		--admin-color-danger-text: color-mix(in srgb, var(--poodle-color-status-danger) 64%, white);
		--admin-color-info: color-mix(in srgb, var(--poodle-color-accent-base) 72%, #3b82f6);
		--admin-color-info-subtle: color-mix(in srgb, var(--admin-color-info) 18%, transparent);

		/* Underlay token mappings */
		--underlay-color-bg-surface: var(--admin-color-surface);
		--underlay-color-text: var(--admin-color-text);
		--underlay-color-surface: var(--admin-color-surface-card);
		--underlay-color-surface-muted: var(--admin-color-surface-card);
		--underlay-color-surface-subtle: var(--admin-color-surface-subtle);
		--underlay-color-surface-hover: var(--admin-color-surface-subtle);
		--underlay-color-on-surface: var(--admin-color-text);
		--underlay-color-primary: var(--admin-color-accent);
		--underlay-color-primary-strong: color-mix(in srgb, var(--admin-color-accent) 72%, black);
		--underlay-color-primary-subtle: color-mix(in srgb, var(--admin-color-accent) 18%, transparent);
		--underlay-color-accent: var(--admin-color-accent);
		--underlay-color-button-neutral-bg: color-mix(in srgb, var(--admin-color-surface-subtle) 84%, black);
		--underlay-color-button-neutral-hover: color-mix(in srgb, var(--admin-color-surface-subtle) 92%, white);
		--underlay-color-on-primary: #ffffff;
		--underlay-color-border: var(--admin-color-border-subtle);
		--underlay-color-border-subtle: var(--admin-color-border-subtle);
		--underlay-color-border-strong: var(--admin-color-border-strong);
		--underlay-color-border-hover: var(--admin-color-border-strong);
		--underlay-color-popover-bg: color-mix(in srgb, var(--admin-color-surface-card) 88%, black);
		--underlay-color-success: var(--admin-color-success);
		--underlay-color-warning: var(--admin-color-warning);
		--underlay-color-danger: var(--admin-color-danger);
		--underlay-color-error: var(--admin-color-danger);
		--underlay-shadow-popover: 0 22px 60px color-mix(in srgb, black 72%, transparent);
		--underlay-color-field-bg: color-mix(in srgb, var(--poodle-color-background-surface) 78%, transparent);
		--underlay-color-text-secondary: var(--poodle-color-text-secondary);
		--underlay-color-text-muted: var(--admin-color-text-muted);
		--underlay-color-surface-raised: var(--admin-color-surface-subtle);

		/* DataTable specific tokens */
		--underlay-table-border: 1px solid var(--admin-color-border-subtle);
		--underlay-table-header-bg: var(--admin-color-surface-subtle);
		--underlay-table-row-hover: color-mix(in srgb, var(--poodle-color-text-secondary) 8%, transparent);
		--underlay-table-row-selected: color-mix(in srgb, var(--admin-color-accent) 12%, transparent);
		--underlay-table-stripe: var(--admin-color-surface-subtle);
		--color-surface: var(--admin-color-surface-card);
		--color-border: var(--admin-color-border-subtle);
		--color-text: var(--admin-color-text);
		--color-text-muted: var(--underlay-color-text-muted);
		--color-primary: var(--admin-color-accent);
		--color-danger: var(--admin-color-danger);
		--radius-lg: 0.5rem;
		--radius-md: 0.375rem;
		--radius-sm: 0.25rem;
	}

	:global(.admin-theme-shell) {
		min-height: 100vh;
	}

	:global(html) {
		font-size: 112.5%;
	}

	:global(body) {
		margin: 0;
		padding: 0;
		min-height: 100vh;
		background: var(--poodle-color-background-canvas);
		color: var(--poodle-color-text-primary);
		font-family:
			"Avenir Next",
			Avenir,
			system-ui,
			-ui-sans-serif,
			-apple-system,
			BlinkMacSystemFont,
			"Segoe UI",
			sans-serif;
	}
</style>
