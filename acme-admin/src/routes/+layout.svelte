<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import "@decodelabs/underlay/styles/base.css";
	// Import Underlay CSS FIRST so our :root overrides take precedence.
	import "@decodelabs/underlay/styles/tokens.css";
	import "@decodelabs/underlay/styles/forms.css";

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children()}

<style>
	:global(:root) {
		/* Core colors */
		--admin-color-bg: #2a3037;
		--admin-color-surface: #050608;
		--admin-color-surface-subtle: #16181d;
		--admin-color-surface-card: #101318;
		--admin-color-border-subtle: rgba(255, 255, 255, 0.1);
		--admin-color-border-strong: rgba(148, 163, 184, 0.6);
		--admin-color-text: #d1d5db;
		--admin-color-text-muted: #9ca3af;
		--admin-color-accent: #14b8a6;

		/* Status colors */
		--admin-color-success: #10b981;
		--admin-color-success-subtle: rgba(16, 185, 129, 0.15);
		--admin-color-warning: #f59e0b;
		--admin-color-warning-subtle: rgba(245, 158, 11, 0.15);
		--admin-color-danger: #ef4444;
		--admin-color-danger-subtle: rgba(239, 68, 68, 0.15);
		--admin-color-danger-text: #fca5a5;
		--admin-color-info: #3b82f6;
		--admin-color-info-subtle: rgba(59, 130, 246, 0.15);

		/* Underlay token mappings */
		--underlay-color-bg-surface: var(--admin-color-surface);
		--underlay-color-text: var(--admin-color-text);
		--underlay-color-text-muted: var(--admin-color-text-muted);
		--underlay-color-surface: var(--admin-color-surface-card);
		--underlay-color-surface-muted: var(--admin-color-surface-card);
		--underlay-color-surface-subtle: var(--admin-color-surface-subtle);
		--underlay-color-surface-hover: var(--admin-color-surface-subtle);
		--underlay-color-on-surface: var(--admin-color-text);
		--underlay-color-primary: var(--admin-color-accent);
		--underlay-color-primary-strong: #0f766e;
		--underlay-color-primary-subtle: rgba(20, 184, 166, 0.15);
		--underlay-color-accent: var(--admin-color-accent);
		--underlay-color-button-neutral-bg: #1f2933;
		--underlay-color-button-neutral-hover: #374151;
		--underlay-color-on-primary: #ffffff;
		--underlay-color-border: var(--admin-color-border-subtle);
		--underlay-color-border-subtle: var(--admin-color-border-subtle);
		--underlay-color-border-strong: var(--admin-color-border-strong);
		--underlay-color-border-hover: var(--admin-color-border-strong);
		--underlay-color-popover-bg: #1f2428;
		--underlay-color-success: var(--admin-color-success);
		--underlay-color-warning: var(--admin-color-warning);
		--underlay-color-danger: var(--admin-color-danger);
		--underlay-color-error: var(--admin-color-danger);
		--underlay-shadow-popover: 0 22px 60px rgba(0, 0, 0, 0.7);

		/* DataTable specific tokens */
		--underlay-table-border: 1px solid var(--admin-color-border-subtle);
		--underlay-table-header-bg: var(--admin-color-surface-subtle);
		--underlay-table-row-hover: rgba(148, 163, 184, 0.08);
		--underlay-table-row-selected: rgba(20, 184, 166, 0.12);
		--underlay-table-stripe: var(--admin-color-surface-subtle);
		--color-surface: var(--admin-color-surface-card);
		--color-border: var(--admin-color-border-subtle);
		--color-text: var(--admin-color-text);
		--color-text-muted: var(--admin-color-text-muted);
		--color-primary: var(--admin-color-accent);
		--color-danger: var(--admin-color-danger);
		--radius-lg: 0.5rem;
		--radius-md: 0.375rem;
		--radius-sm: 0.25rem;
	}

	:global(html) {
		font-size: 112.5%;
	}

	:global(body) {
		margin: 0;
		padding: 0;
		background:
			radial-gradient(900px 520px at 20% -10%, rgba(20, 184, 166, 0.18), transparent),
			linear-gradient(180deg, #13171f, var(--admin-color-bg));
		color: var(--admin-color-text);
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
