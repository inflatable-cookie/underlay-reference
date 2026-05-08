<script lang="ts">
	import User from "lucide-svelte/icons/user";
	import LogOut from "lucide-svelte/icons/log-out";
	import {
		AlertDialog as PoodleAlertDialog,
		Popover as PoodlePopover
	} from "@poodle/svelte";
	import { auth } from "$lib/stores/auth";
	import type { LoginUser } from "@api-client";

	interface Props {
		currentUser?: LoginUser | null;
		variant?: "desktop" | "mobile";
		onNavigate?: () => void;
	}

	let {
		currentUser = null,
		variant = "desktop",
		onNavigate
	}: Props = $props();

	let userMenuOpen = $state(false);
	let logoutOpen = $state(false);
	let loggingOut = $state(false);

	const initialsFromName = (name: string) => {
		const parts = name
			.split(/\s+/)
			.map((p) => p.trim())
			.filter(Boolean);

		if (parts.length === 0) return "??";

		const first = parts[0]?.[0] ?? "";
		const second = parts.length > 1 ? parts[1]?.[0] ?? "" : parts[0]?.[1] ?? "";
		return (first + second).toUpperCase() || "??";
	};

	const roleLabel = (roles: string[] | undefined) => {
		const role = roles?.[0] ?? "";
		if (!role) return "Staff";
		return role.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
	};

	const confirmLogout = async () => {
		loggingOut = true;
		try {
			await auth.logout();
		} finally {
			loggingOut = false;
			logoutOpen = false;
		}
	};

	const handleNavigate = () => {
		userMenuOpen = false;
		onNavigate?.();
	};

	const handleLogoutClick = () => {
		userMenuOpen = false;
		onNavigate?.();
		logoutOpen = true;
	};
</script>

<div class={variant === "mobile" ? "admin-mobile-overlay__user" : "admin-nav__user-section"}>
	<PoodlePopover bind:open={userMenuOpen} placement="top-start" ariaLabel="User menu" block>
		<div slot="trigger" class="admin-nav__user-trigger">
			<div class="admin-nav__user-avatar" aria-hidden="true">
				{initialsFromName(currentUser?.displayName ?? "Admin")}
			</div>
			<div class="admin-nav__user-meta">
				<span class="admin-nav__user-name">{currentUser?.displayName ?? "Admin user"}</span>
				<span class="admin-nav__user-role">{roleLabel(currentUser?.roles)}</span>
			</div>
		</div>

		<nav class="admin-nav__user-menu" aria-label="User menu">
			<a href="/account" class="admin-nav__user-menu-item" onclick={handleNavigate}>
				<User class="admin-nav__user-menu-icon" />
				<span>Account</span>
			</a>
			<button
				type="button"
				class="admin-nav__user-menu-item admin-nav__user-menu-item--danger"
				onclick={handleLogoutClick}
				disabled={loggingOut}
			>
				<LogOut class="admin-nav__user-menu-icon" />
				<span>Log out</span>
			</button>
		</nav>
	</PoodlePopover>

	<PoodleAlertDialog
		bind:open={logoutOpen}
		title="Log out?"
		description="You'll be signed out of Acme Admin."
		confirmLabel={loggingOut ? "Logging out..." : "Log out"}
		cancelLabel="Cancel"
		onConfirm={confirmLogout}
		onCancel={() => (logoutOpen = false)}
		tone="danger"
	/>
</div>

<style>
	:global(.admin-nav__user-section) {
		margin-top: 1rem;
		padding: 1rem 0.9rem 0 0;
		border-top: 1px solid var(--admin-color-border-subtle);
		flex-shrink: 0;
	}

	:global(.admin-nav__user-trigger) {
		display: flex;
		align-items: center;
		width: 100%;
		padding: 0.5rem;
		border-radius: 0.5rem;
		background: transparent;
		border: 1px solid transparent;
		justify-content: flex-start;
		gap: 0.6rem;
	}

	:global(.admin-nav__user-trigger:hover) {
		background: rgba(148, 163, 184, 0.16);
	}

	:global(.admin-nav__user-trigger[data-state="open"]) {
		background: rgba(148, 163, 184, 0.12);
		border-color: var(--admin-color-border-subtle);
	}

	:global(.admin-nav__user-avatar) {
		width: 2rem;
		height: 2rem;
		border-radius: 999px;
		background: linear-gradient(135deg, #22c55e, #14b8a6);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 0.75rem;
		font-weight: 700;
		color: #020617;
		flex-shrink: 0;
	}

	:global(.admin-nav__user-meta) {
		display: flex;
		flex-direction: column;
		gap: 0.1rem;
		text-align: left;
		min-width: 0;
	}

	:global(.admin-nav__user-name) {
		font-size: 0.85rem;
		font-weight: 600;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	:global(.admin-nav__user-role) {
		font-size: 0.7rem;
		color: var(--admin-color-text-muted);
	}

	:global(.admin-nav__user-menu) {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		width: 100%;
		min-width: 0;
	}

	:global(.admin-nav__user-section .poodle-popover),
	:global(.admin-mobile-overlay__user .poodle-popover) {
		width: 100%;
	}

	:global(.admin-nav__user-section .poodle-popover__surface),
	:global(.admin-mobile-overlay__user .poodle-popover__surface) {
		width: 100%;
		min-width: 100%;
		box-sizing: border-box;
	}

	:global(.admin-nav__user-menu-item) {
		display: flex;
		align-items: center;
		gap: 0.6rem;
		padding: 0.5rem 0.65rem;
		border-radius: 0.4rem;
		background: transparent;
		border: none;
		color: inherit;
		font-size: 0.85rem;
		text-decoration: none;
		cursor: pointer;
		width: 100%;
		box-sizing: border-box;
		text-align: left;
	}

	:global(.admin-nav__user-menu-item:hover) {
		background: rgba(148, 163, 184, 0.16);
	}

	:global(.admin-nav__user-menu-item--danger) {
		color: #f87171;
	}

	:global(.admin-nav__user-menu-item--danger:hover) {
		background: rgba(239, 68, 68, 0.15);
	}

	:global(.admin-nav__user-menu-icon) {
		width: 1rem;
		height: 1rem;
		flex-shrink: 0;
	}

	:global(.admin-mobile-overlay__user) {
		margin-top: auto;
		padding-top: 1rem;
		border-top: 1px solid var(--admin-color-border-subtle);
	}
</style>
