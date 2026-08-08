<script lang="ts">
	import { page } from '$app/stores';
	import favicon from '$lib/assets/favicon.svg';
	import { resolveAdminPageTitle } from '$lib/utils/page-title';
	import "@inflatable-cookie/poodle-svelte-tokens/styles.css";
	import "@inflatable-cookie/poodle-svelte-tokens/theme-eclipse.css";
	import "@inflatable-cookie/poodle-svelte-tokens/density-compact.css";
	import "@inflatable-cookie/poodle-svelte-tokens/control-size-md.css";
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
	data-theme="eclipse"
	data-density="compact"
	data-control-size="md"
>
	{@render children()}
</div>

<style>
	:global(.admin-theme-shell) {
		/* Core colors */
		--admin-color-bg: var(--poodle-color-background-canvas);
		--admin-color-surface: var(--poodle-color-background-panel);
		--admin-color-surface-subtle: var(--poodle-color-background-surface);
		--admin-color-surface-card: var(--poodle-color-background-surface);
		--admin-color-border-subtle: var(--poodle-color-border-subtle);
		--admin-color-border-strong: var(--poodle-color-border-default);
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
		--admin-color-danger-text: var(--poodle-color-status-danger);
		--admin-color-info: var(--poodle-color-accent-base);
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
		--underlay-color-primary-strong: color-mix(in srgb, var(--admin-color-accent) 82%, black);
		--underlay-color-primary-subtle: color-mix(in srgb, var(--admin-color-accent) 18%, transparent);
		--underlay-color-accent: var(--admin-color-accent);
		--underlay-color-button-neutral-bg: var(--admin-color-surface-subtle);
		--underlay-color-button-neutral-hover: color-mix(in srgb, var(--admin-color-surface-subtle) 92%, white);
		--underlay-color-on-primary: #ffffff;
		--underlay-color-border: var(--admin-color-border-subtle);
		--underlay-color-border-subtle: var(--admin-color-border-subtle);
		--underlay-color-border-strong: var(--admin-color-border-strong);
		--underlay-color-border-hover: var(--admin-color-border-strong);
		--underlay-color-popover-bg: var(--admin-color-surface-card);
		--underlay-color-success: var(--admin-color-success);
		--underlay-color-warning: var(--admin-color-warning);
		--underlay-color-danger: var(--admin-color-danger);
		--underlay-color-error: var(--admin-color-danger);
		--underlay-shadow-popover: var(--poodle-shadow-lg);
		--underlay-color-field-bg: var(--poodle-color-background-surface);
		--underlay-color-text-secondary: var(--poodle-color-text-secondary);
		--underlay-color-text-muted: var(--admin-color-text-muted);
		--underlay-color-surface-raised: var(--admin-color-surface-subtle);

		/* DataTable specific tokens */
		--underlay-table-border: 1px solid var(--admin-color-border-subtle);
		--underlay-table-header-bg: var(--admin-color-surface-subtle);
		--underlay-table-row-hover: color-mix(in srgb, var(--poodle-color-text-secondary) 6%, transparent);
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
		--poodle-surface: var(--poodle-color-background-canvas);
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
		font-family: var(--poodle-typography-font-family-sans);
		--poodle-surface: var(--poodle-color-background-canvas);
	}
</style>
