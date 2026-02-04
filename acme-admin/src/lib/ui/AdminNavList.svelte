<script lang="ts">
	import ChevronDown from "lucide-svelte/icons/chevron-down";
	import Gauge from "lucide-svelte/icons/gauge";
	import Box from "lucide-svelte/icons/box";
	import Settings from "lucide-svelte/icons/settings";
	import Users from "lucide-svelte/icons/users";
	import Image from "lucide-svelte/icons/image";
	// Sub-link icons
	import Tags from "lucide-svelte/icons/tags";
	import FolderKanban from "lucide-svelte/icons/folder-kanban";
	import Images from "lucide-svelte/icons/images";
	import Upload from "lucide-svelte/icons/upload";
	import Trash2 from "lucide-svelte/icons/trash-2";
	import Layers from "lucide-svelte/icons/layers";
	import Calendar from "lucide-svelte/icons/calendar";
	import AlertTriangle from "lucide-svelte/icons/alert-triangle";
	import Mail from "lucide-svelte/icons/mail";

	interface Props {
		currentSection?: string | null;
		currentPath?: string;
		onNavigate?: () => void;
		variant?: "desktop" | "mobile";
	}

	let {
		currentSection = null,
		currentPath = "",
		onNavigate,
		variant = "desktop"
	}: Props = $props();

	let expandedSection = $state<string | null>(null);

	$effect(() => {
		if (currentSection) {
			expandedSection = currentSection;
		}
	});

	const handleChildClick = () => {
		onNavigate?.();
	};

	const toggleSection = (section: string) => {
		expandedSection = expandedSection === section ? null : section;
	};

	function isActive(href: string): boolean {
		if (!currentPath) return false;
		if (href === "/") return currentPath === "/";
		return currentPath === href || currentPath.startsWith(href + "/");
	}

	let linkClass = $derived(variant === "mobile" ? "admin-mobile-overlay__link" : "admin-nav__root");
	let childrenClass = $derived(
		variant === "mobile" ? "admin-mobile-overlay__children" : "admin-nav__children"
	);
	let sectionClass = $derived(variant === "desktop" ? "admin-nav__section" : "");
</script>

<ul class={variant === "mobile" ? "admin-mobile-overlay__list" : "admin-nav__list"}>
	<li>
		<a href="/" class="{linkClass} admin-nav__root" class:admin-nav__link--active={isActive("/")} onclick={handleChildClick}>
			<span class="admin-nav__badge admin-nav__badge--overview" aria-hidden="true">
				<Gauge class="admin-nav__badge-icon" />
			</span>
			<span class={variant === "desktop" ? "admin-nav__label" : ""}>Dashboard</span>
		</a>
	</li>

	<li>
		<button
			type="button"
			class="{linkClass} {sectionClass} admin-nav__section-toggle"
			aria-expanded={expandedSection === "acme"}
			onclick={() => toggleSection("acme")}
		>
			<span class="admin-nav__badge admin-nav__badge--acme" aria-hidden="true">
				<Box class="admin-nav__badge-icon" />
			</span>
			<span class={variant === "desktop" ? "admin-nav__label" : ""}>Acme</span>
			<ChevronDown class="admin-nav__chevron" />
		</button>
		{#if expandedSection === "acme"}
			<ul class={childrenClass}>
				<li>
					<a href="/categories" class:admin-nav__link--active={isActive("/categories")} onclick={handleChildClick}>
						<Tags class="admin-nav__child-icon" />
						Categories
					</a>
				</li>
				<li>
					<a href="/projects" class:admin-nav__link--active={isActive("/projects")} onclick={handleChildClick}>
						<FolderKanban class="admin-nav__child-icon" />
						Projects
					</a>
				</li>
			</ul>
		{/if}
	</li>

	<li>
		<button
			type="button"
			class="{linkClass} {sectionClass} admin-nav__section-toggle"
			aria-expanded={expandedSection === "media"}
			onclick={() => toggleSection("media")}
		>
			<span class="admin-nav__badge admin-nav__badge--media" aria-hidden="true">
				<Image class="admin-nav__badge-icon" />
			</span>
			<span class={variant === "desktop" ? "admin-nav__label" : ""}>Media</span>
			<ChevronDown class="admin-nav__chevron" />
		</button>
		{#if expandedSection === "media"}
			<ul class={childrenClass}>
				<li>
					<a href="/media" class:admin-nav__link--active={isActive("/media") && !isActive("/media/upload") && !isActive("/media/trash")} onclick={handleChildClick}>
						<Images class="admin-nav__child-icon" />
						Library
					</a>
				</li>
				<li>
					<a href="/media/upload" class:admin-nav__link--active={isActive("/media/upload")} onclick={handleChildClick}>
						<Upload class="admin-nav__child-icon" />
						Upload
					</a>
				</li>
				<li>
					<a href="/media/trash" class="admin-nav__child-link--danger" class:admin-nav__link--active={isActive("/media/trash")} onclick={handleChildClick}>
						<Trash2 class="admin-nav__child-icon" />
						Trash
					</a>
				</li>
			</ul>
		{/if}
	</li>

	<li>
		<a href="/users" class="{linkClass} admin-nav__root" class:admin-nav__link--active={isActive("/users")} onclick={handleChildClick}>
			<span class="admin-nav__badge admin-nav__badge--users" aria-hidden="true">
				<Users class="admin-nav__badge-icon" />
			</span>
			<span class={variant === "desktop" ? "admin-nav__label" : ""}>Users</span>
		</a>
	</li>

	<li>
		<button
			type="button"
			class="{linkClass} {sectionClass} admin-nav__section-toggle"
			aria-expanded={expandedSection === "system"}
			onclick={() => toggleSection("system")}
		>
			<span class="admin-nav__badge admin-nav__badge--system" aria-hidden="true">
				<Settings class="admin-nav__badge-icon" />
			</span>
			<span class={variant === "desktop" ? "admin-nav__label" : ""}>System</span>
			<ChevronDown class="admin-nav__chevron" />
		</button>
		{#if expandedSection === "system"}
			<ul class={childrenClass}>
				<li>
					<a href="/system/jobs" class:admin-nav__link--active={isActive("/system/jobs")} onclick={handleChildClick}>
						<Layers class="admin-nav__child-icon" />
						Jobs
					</a>
				</li>
				<li>
					<a href="/system/scheduled-tasks" class:admin-nav__link--active={isActive("/system/scheduled-tasks")} onclick={handleChildClick}>
						<Calendar class="admin-nav__child-icon" />
						Scheduled Tasks
					</a>
				</li>
				<li>
					<a href="/system/errors" class:admin-nav__link--active={isActive("/system/errors")} onclick={handleChildClick}>
						<AlertTriangle class="admin-nav__child-icon" />
						Errors
					</a>
				</li>
				<li>
					<a href="/system/emails" class:admin-nav__link--active={isActive("/system/emails")} onclick={handleChildClick}>
						<Mail class="admin-nav__child-icon" />
						Emails
					</a>
				</li>
			</ul>
		{/if}
	</li>
</ul>

<style>
	:global(.admin-nav__list) {
		list-style: none;
		padding: 0 0.9rem 0 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		flex: 1;
		overflow-y: auto;
		min-height: 0;
	}

	:global(.admin-nav__root),
	:global(.admin-nav__section) {
		display: flex;
		align-items: center;
		box-sizing: border-box;
		padding: 0.4rem 0.7rem;
		border-radius: 0.4rem;
		color: inherit;
		text-decoration: none;
		font-size: 0.9rem;
		border: none;
		background: transparent;
		width: 100%;
		cursor: pointer;
		text-align: left;
	}

	:global(.admin-nav__root:hover),
	:global(.admin-nav__section-toggle:hover) {
		background-color: rgba(148, 163, 184, 0.16);
	}

	:global(.admin-nav__section-toggle) {
		opacity: 0.7;
		transition: opacity 0.15s ease;
	}

	:global(.admin-nav__section-toggle:hover),
	:global(.admin-nav__section-toggle[aria-expanded="true"]) {
		opacity: 1;
	}

	:global(.admin-nav__chevron) {
		width: 1rem;
		height: 1rem;
		margin-left: auto;
		opacity: 0.5;
		transition: transform 0.15s ease;
		flex-shrink: 0;
	}

	:global(.admin-nav__section-toggle[aria-expanded="true"] .admin-nav__chevron) {
		transform: rotate(180deg);
	}

	:global(.admin-nav__children) {
		list-style: none;
		padding-left: 1.25rem;
		margin: 0.25rem 0 0.4rem;
		display: flex;
		flex-direction: column;
		gap: 0.2rem;
	}

	:global(.admin-nav__children a) {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.85rem;
		padding: 0.45rem 0.75rem;
		border-radius: 0.4rem;
		opacity: 0.7;
		transition: opacity 0.15s ease;
		color: inherit;
		text-decoration: none;
	}

	:global(.admin-nav__children a:hover),
	:global(.admin-nav__children a.admin-nav__link--active) {
		opacity: 1;
		background-color: rgba(148, 163, 184, 0.16);
	}

	:global(.admin-nav__child-icon) {
		width: 0.9rem;
		height: 0.9rem;
		flex-shrink: 0;
	}

	:global(.admin-nav__child-link--danger) {
		color: #f87171;
	}

	:global(.admin-nav__child-link--danger:hover),
	:global(.admin-nav__child-link--danger.admin-nav__link--active) {
		color: #ef4444;
		background-color: rgba(239, 68, 68, 0.12);
	}

	:global(.admin-nav__link--active) {
		background-color: rgba(148, 163, 184, 0.12);
		font-weight: 550;
	}

	:global(.admin-nav__badge) {
		min-width: 1.5rem;
		min-height: 1.5rem;
		padding: 0.05rem;
		border-radius: 0.45rem;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		margin-right: 0.55rem;
	}

	:global(.admin-nav__badge-icon) {
		width: 0.95rem;
		height: 0.95rem;
		color: #f9fafb;
	}

	:global(.admin-nav__badge--overview) {
		background: linear-gradient(135deg, #22c55e, #14b8a6);
	}

	:global(.admin-nav__badge--users) {
		background: linear-gradient(135deg, #3b82f6, #8b5cf6);
	}

	:global(.admin-nav__badge--acme) {
		background: linear-gradient(135deg, #8b5cf6, #6366f1);
	}

	:global(.admin-nav__badge--media) {
		background: linear-gradient(135deg, #f59e0b, #f97316);
	}

	:global(.admin-nav__badge--system) {
		background: linear-gradient(135deg, #94a3b8, #64748b);
	}

	:global(.admin-mobile-overlay__list) {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		flex: 1;
	}

	:global(.admin-mobile-overlay__link) {
		display: flex;
		align-items: center;
		gap: 0.55rem;
		padding: 0.6rem 0.75rem;
		border-radius: 0.5rem;
		color: inherit;
		text-decoration: none;
		font-size: 1rem;
		border: none;
		background: transparent;
		width: 100%;
		cursor: pointer;
		text-align: left;
		opacity: 0.75;
		transition: opacity 0.15s ease;
	}

	:global(.admin-mobile-overlay__link:hover),
	:global(.admin-mobile-overlay__link.admin-nav__link--active),
	:global(.admin-mobile-overlay__link[aria-expanded="true"]) {
		opacity: 1;
	}

	:global(.admin-mobile-overlay__link:hover) {
		background: rgba(148, 163, 184, 0.16);
	}

	:global(.admin-mobile-overlay__children) {
		list-style: none;
		padding: 0.25rem 0 0.5rem 2.5rem;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 0.2rem;
	}

	:global(.admin-mobile-overlay__children a) {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 0.75rem;
		border-radius: 0.4rem;
		color: inherit;
		text-decoration: none;
		font-size: 0.95rem;
		opacity: 0.7;
		transition: opacity 0.15s ease;
	}

	:global(.admin-mobile-overlay__children a:hover),
	:global(.admin-mobile-overlay__children a.admin-nav__link--active) {
		background: rgba(148, 163, 184, 0.16);
		opacity: 1;
	}

	:global(.admin-mobile-overlay__children .admin-nav__child-link--danger) {
		color: #f87171;
	}

	:global(.admin-mobile-overlay__children .admin-nav__child-link--danger:hover),
	:global(.admin-mobile-overlay__children .admin-nav__child-link--danger.admin-nav__link--active) {
		color: #ef4444;
		background: rgba(239, 68, 68, 0.12);
	}

	:global(.admin-mobile-overlay__link .admin-nav__chevron) {
		width: 1.25rem;
		height: 1.25rem;
	}

	:global(.admin-nav__label) {
		line-height: 1.2;
	}
</style>
