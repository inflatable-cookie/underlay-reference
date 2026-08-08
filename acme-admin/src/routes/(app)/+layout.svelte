<script lang="ts">
import {
  configureAuth
} from "@inflatable-cookie/underlay/runtime/auth";
import {
  UNDERLAY_TOASTS_CONTEXT_KEY,
  createToastStore,
} from "@inflatable-cookie/underlay/runtime/feedback";
import {
  setContext,
  onMount } from "svelte";
	import { goto } from "$app/navigation";
	import { page } from "$app/stores";
	import { ToastHost } from "@inflatable-cookie/poodle-svelte";
	import { Drawer } from "@inflatable-cookie/poodle-svelte";
	import Menu from "lucide-svelte/icons/menu";
	import X from "lucide-svelte/icons/x";
	import PanelRight from "lucide-svelte/icons/panel-right";
		import { auth, authLoading, currentUser } from "$lib/stores/auth";
	import AdminNavList from "$lib/ui/AdminNavList.svelte";
	import AdminUserMenu from "$lib/ui/AdminUserMenu.svelte";
	import { ErrorBoundary as AdminErrorBoundary } from "@inflatable-cookie/poodle-svelte";

	// Configure global auth handlers for useAuthenticatedData
	// Enables automatic token refresh on 401 errors and auto-fetch on auth readiness
	configureAuth({
		getToken: () => auth.getToken(),
		onRefresh: auth.getRefreshHandler(),
		getAuthLoading: () => $authLoading,
		getCurrentUser: () => $currentUser
	});

	let { children } = $props();

	const toastStore = createToastStore();
	setContext(UNDERLAY_TOASTS_CONTEXT_KEY, toastStore);

	let mobileMenuOpen = $state(false);
	let contextPanelOpen = $state(false);
	let contextPanelIsMobile = $state(false);

	const closeMobileMenu = () => {
		mobileMenuOpen = false;
	};

	const toggleContextPanel = () => {
		contextPanelOpen = !contextPanelOpen;
	};

	const closeContextPanel = () => {
		contextPanelOpen = false;
	};

	const setContextPanelOpen = (nextOpen: boolean) => {
		contextPanelOpen = nextOpen;
	};

	const currentSection = $derived.by(() => {
		const path = $page.url.pathname;
		if (path.startsWith("/categories") || path.startsWith("/projects")) return "acme";
		if (path.startsWith("/media")) return "media";
		if (path.startsWith("/system")) return "system";
		return "overview";
	});

	// Initialize auth on mount
	onMount(() => {
		auth.initialize();

		const mediaQuery = window.matchMedia("(max-width: 900px)");
		const updateContextPanelViewport = () => {
			contextPanelIsMobile = mediaQuery.matches;
		};

		updateContextPanelViewport();
		mediaQuery.addEventListener("change", updateContextPanelViewport);

		return () => {
			mediaQuery.removeEventListener("change", updateContextPanelViewport);
		};
	});

	// Redirect to login if not authenticated after initialization
	$effect(() => {
		if (!$authLoading && !$currentUser) {
			goto('/login');
		}
	});

</script>

{#if $authLoading}
	<div class="admin-loading">
		<div class="admin-loading__spinner"></div>
		<p class="admin-loading__text">Loading...</p>
	</div>
{:else if $currentUser}
<div class="admin-app-shell">
	<header class="admin-mobile-header">
		<a href="/" class="admin-mobile-header__brand">
			<span class="admin-mobile-header__title">Acme</span>
			<span class="admin-mobile-header__env">Admin</span>
		</a>
		<div class="admin-mobile-header__actions">
			<button
				type="button"
				class="admin-mobile-header__icon-btn"
				aria-label={contextPanelOpen ? "Close context panel" : "Open context panel"}
				onclick={toggleContextPanel}
			>
				<PanelRight class="admin-mobile-header__icon" />
			</button>
			<button
				type="button"
				class="admin-mobile-header__icon-btn"
				aria-label={mobileMenuOpen ? "Close menu" : "Open menu"}
				aria-expanded={mobileMenuOpen}
				onclick={() => (mobileMenuOpen = !mobileMenuOpen)}
			>
				{#if mobileMenuOpen}
					<X class="admin-mobile-header__icon" />
				{:else}
					<Menu class="admin-mobile-header__icon" />
				{/if}
			</button>
		</div>
	</header>

	{#if mobileMenuOpen}
		<div class="admin-mobile-overlay" role="dialog" aria-modal="true" aria-label="Navigation menu">
			<nav class="admin-mobile-overlay__nav" aria-label="Main">
				<AdminNavList
					currentSection={currentSection}
					currentPath={$page.url.pathname}
					onNavigate={closeMobileMenu}
					variant="mobile"
				/>
				<AdminUserMenu
					currentUser={$currentUser}
					variant="mobile"
					onNavigate={closeMobileMenu}
				/>
			</nav>
		</div>
	{/if}

	<div class="admin-main">
		<nav class="admin-nav" aria-label="Main">
			<div class="admin-nav__inner">
				<a href="/" class="admin-nav__brand">
					<span class="admin-nav__title">Acme</span>
					<span class="admin-nav__env">Admin</span>
				</a>
				<AdminNavList currentSection={currentSection} currentPath={$page.url.pathname} variant="desktop" />
				<AdminUserMenu currentUser={$currentUser} variant="desktop" />
			</div>
		</nav>

		<div class="admin-content">
			<main class="admin-content__body">
				<AdminErrorBoundary>
					{@render children()}
				</AdminErrorBoundary>
			</main>
		</div>

		{#if !contextPanelIsMobile}
			<button
				type="button"
				class="admin-context-panel__toggle-strip"
				aria-label={contextPanelOpen ? "Close context panel" : "Open context panel"}
				aria-pressed={contextPanelOpen}
				onclick={toggleContextPanel}
			>
				<PanelRight class="admin-context-panel__toggle-icon" />
			</button>
		{/if}
	</div>

	<Drawer
		open={contextPanelOpen}
		edge="right"
		modal={contextPanelIsMobile}
		ariaLabel="Context panel"
		onRequestClose={closeContextPanel}
		onOpenChange={(open) => {
			setContextPanelOpen(open);
		}}
	>
		<div class="admin-context-panel__content">
			<div class="admin-context-panel__header">
				<h2 class="admin-context-panel__title">Context</h2>
				<button
					type="button"
					class="admin-context-panel__close"
					aria-label="Close context panel"
					onclick={closeContextPanel}
				>
					<X class="admin-context-panel__close-icon" />
				</button>
			</div>
			<div class="admin-context-panel__body">
				<p class="admin-context-panel__placeholder">
					Context actions and assistive tools live here.
				</p>
			</div>
		</div>
	</Drawer>

	<ToastHost store={toastStore} />
</div>
{/if}

<style>
	/* Loading state */
	.admin-loading {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100vh;
		gap: 1rem;
	}

	.admin-loading__spinner {
		width: 2rem;
		height: 2rem;
		border: 2px solid var(--admin-color-border-subtle);
		border-top-color: var(--admin-color-accent);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.admin-loading__text {
		color: var(--admin-color-text-muted);
		font-size: 0.9rem;
	}

	:global(body) {
		padding: 0;
		overflow: hidden;
		height: 100vh;
	}

	.admin-app-shell {
		height: 100vh;
		width: 100%;
		display: flex;
		flex-direction: column;
		max-width: none;
		margin: 0;
		border-radius: 0;
		overflow: hidden;
		background: var(--admin-color-surface);
		box-shadow: none;
		border: none;
	}

	.admin-main {
		flex: 1;
		display: grid;
		grid-template-columns: 304px 1fr;
		grid-template-areas: "nav content";
		min-height: 0;
		overflow: hidden;
		position: relative;
	}

	.admin-nav {
		grid-area: nav;
		border-right: 1px solid rgba(15, 23, 42, 0.9);
		padding: 1.1rem 0 1.1rem 0.9rem;
		background-color: var(--admin-color-surface-subtle);
		overflow: hidden;
	}

	.admin-nav__inner {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.admin-nav__brand {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.2rem 0.9rem 1rem 0.5rem;
		flex-shrink: 0;
		text-decoration: none;
		color: inherit;
	}

	.admin-nav__title {
		font-weight: 650;
		letter-spacing: 0.04em;
	}

	.admin-nav__env {
		font-size: 0.75rem;
		padding: 0.1rem 0.35rem;
		border-radius: 999px;
		border: 1px solid var(--admin-color-border-strong);
		text-transform: uppercase;
		color: var(--admin-color-text);
		opacity: 0.9;
	}

	.admin-content {
		grid-area: content;
		padding: 1.75rem 2.25rem;
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		overflow-y: auto;
	}

	.admin-content__body {
		flex: 1;
	}

	.admin-context-panel__content {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
	}

	.admin-context-panel__toggle-strip {
		position: absolute;
		right: 0;
		top: 0;
		bottom: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 55px;
		padding: 0;
		border: none;
		border-left: 1px solid rgba(15, 23, 42, 0.9);
		border-right: 1px solid rgba(15, 23, 42, 0.9);
		background: #0b0d11;
		color: var(--admin-color-text-muted);
		cursor: pointer;
		z-index: 70;
		overflow: hidden;
		transition:
			opacity 0.2s ease,
			background-color 0.15s ease,
			color 0.15s ease,
			border-color 0.15s ease;
	}

	.admin-context-panel__toggle-strip:hover {
		background: rgba(148, 163, 184, 0.12);
		color: var(--admin-color-text);
	}

	.admin-context-panel__toggle-strip:focus-visible {
		outline: 2px solid var(--admin-color-accent);
		outline-offset: -2px;
	}

	.admin-context-panel__toggle-strip[aria-pressed="true"] {
		opacity: 0;
		pointer-events: none;
	}

	.admin-context-panel__header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1rem 1rem 0.75rem;
		flex-shrink: 0;
	}

	.admin-context-panel__title {
		font-size: 0.75rem;
		font-weight: 500;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--admin-color-text-muted);
		opacity: 0.6;
		margin: 0;
	}

	.admin-context-panel__body {
		flex: 1;
		padding: 0 1rem 1rem;
		overflow-y: auto;
	}

	.admin-context-panel__placeholder {
		font-size: 0.85rem;
		color: var(--admin-color-text-muted);
		opacity: 0.5;
	}

	.admin-context-panel__close {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 2rem;
		height: 2rem;
		padding: 0;
		background: transparent;
		border: none;
		border-radius: 0.4rem;
		color: var(--admin-color-text-muted);
		cursor: pointer;
	}

	.admin-context-panel__close:hover {
		background: rgba(148, 163, 184, 0.16);
		color: var(--admin-color-text);
	}

	.admin-context-panel__close:focus-visible {
		outline: 2px solid var(--admin-color-accent);
		outline-offset: 2px;
	}

	:global(.admin-context-panel__toggle-icon) {
		width: 1.25rem;
		height: 1.25rem;
	}

	:global(.admin-context-panel__close-icon) {
		width: 1rem;
		height: 1rem;
	}

	:global(.drawer[data-edge="right"]) {
		padding-right: 0;
		z-index: 65;
	}

	:global(.drawer[data-edge="right"] .drawer__surface) {
		width: min(368px, 100vw);
		height: 100vh;
		margin: 0;
		padding: 0;
		border: 1px solid rgba(15, 23, 42, 0.9);
		border-right: none;
		border-radius: 0;
		background: #0b0d11;
		box-shadow: -8px 0 24px rgba(0, 0, 0, 0.3);
		overflow: hidden;
	}

	@media (min-width: 901px) {
		.admin-content {
			padding-right: calc(2.25rem + 55px);
		}
	}

	.admin-mobile-header {
		display: none;
	}

	.admin-mobile-overlay {
		display: none;
	}

	@media (max-width: 900px) {
		:global(body) {
			padding: 0;
		}

		.admin-app-shell {
			max-width: none;
			margin: 0;
			height: 100vh;
			border-radius: 0;
			box-shadow: none;
			border: none;
		}

		.admin-mobile-header {
			display: flex;
			align-items: center;
			justify-content: space-between;
			padding: 0.75rem 1rem;
			background: var(--admin-color-surface-subtle);
			border-bottom: 1px solid var(--admin-color-border-subtle);
			flex-shrink: 0;
		}

		.admin-mobile-header__brand {
			display: flex;
			align-items: center;
			gap: 0.5rem;
			text-decoration: none;
			color: inherit;
		}

		.admin-mobile-header__title {
			font-weight: 650;
			letter-spacing: 0.04em;
		}

		.admin-mobile-header__env {
			font-size: 0.7rem;
			padding: 0.1rem 0.3rem;
			border-radius: 999px;
			border: 1px solid var(--admin-color-border-strong);
			text-transform: uppercase;
		}

		.admin-mobile-header__actions {
			display: flex;
			align-items: center;
			gap: 0.25rem;
		}

		.admin-mobile-header__icon-btn {
			display: flex;
			align-items: center;
			justify-content: center;
			width: 2.5rem;
			height: 2.5rem;
			padding: 0;
			background: transparent;
			border: none;
			border-radius: 0.5rem;
			color: var(--admin-color-text);
			cursor: pointer;
		}

		.admin-mobile-header__icon-btn:hover {
			background: rgba(148, 163, 184, 0.16);
		}

		.admin-mobile-header__icon-btn:focus-visible {
			outline: 2px solid var(--admin-color-accent);
			outline-offset: 2px;
		}

		:global(.admin-mobile-header__icon) {
			width: 1.5rem;
			height: 1.5rem;
		}

		.admin-mobile-overlay {
			display: flex;
			flex-direction: column;
			position: fixed;
			inset: 0;
			top: 3.5rem;
			background: var(--admin-color-bg);
			z-index: 100;
			overflow-y: auto;
			padding: 1rem;
		}

		.admin-mobile-overlay__nav {
			display: flex;
			flex-direction: column;
			flex: 1;
			padding: 1rem;
			max-width: 420px;
			margin: 0 auto;
			width: 100%;
			box-sizing: border-box;
			background: var(--admin-color-surface-subtle);
			border-radius: 0.75rem;
		}

		.admin-nav {
			display: none;
		}

		.admin-main {
			grid-template-columns: 1fr;
			grid-template-areas: "content";
		}

		.admin-content {
			padding: 1.25rem;
			font-size: 0.95rem;
		}

		.admin-context-panel__toggle-strip {
			display: none;
		}

		:global(.drawer[data-edge="right"]) {
			padding-right: 0;
			z-index: 110;
		}

		:global(.drawer[data-edge="right"] .drawer__backdrop) {
			top: 3.5rem;
			background: rgba(0, 0, 0, 0.5);
		}

		:global(.drawer[data-edge="right"] .drawer__surface) {
			width: min(368px, 85vw);
			height: auto;
			margin: 0;
			border-left: 1px solid var(--admin-color-border-subtle);
			border-top: none;
			border-bottom: none;
			border-radius: 0;
			box-shadow: none;
		}

		.admin-context-panel__close {
			display: flex;
			align-items: center;
			justify-content: center;
			width: 2rem;
			height: 2rem;
			padding: 0;
			background: transparent;
			border: none;
			border-radius: 0.4rem;
			color: var(--admin-color-text-muted);
			cursor: pointer;
		}

		.admin-context-panel__close:hover {
			background: rgba(148, 163, 184, 0.16);
			color: var(--admin-color-text);
		}

		.admin-context-panel__close:focus-visible {
			outline: 2px solid var(--admin-color-accent);
			outline-offset: 2px;
		}

		:global(.admin-context-panel__close-icon) {
			width: 1rem;
			height: 1rem;
		}

	}
</style>
