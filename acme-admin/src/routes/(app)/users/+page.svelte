<script lang="ts">
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";
	import { adminCommands, type User, type UserRole, type UserStatus, UserRole as UserRoleConst, UserStatus as UserStatusConst } from "@api-client";
	import { Card, Pill } from "@decodelabs/underlay/components";
	import ChevronLeft from "lucide-svelte/icons/chevron-left";
	import ChevronRight from "lucide-svelte/icons/chevron-right";
	import Search from "lucide-svelte/icons/search";
	import { auth } from "$lib/stores/auth";

	const PAGE_SIZE = 20;

	let users = $state<User[]>([]);
	let total = $state(0);
	let hasMore = $state(false);
	let loading = $state(true);
	let error = $state<string | null>(null);

	let offset = $state(0);
	let roleFilter = $state<UserRole | "">("");
	let statusFilter = $state<UserStatus | "">("");
	let searchQuery = $state("");
	let searchTimeout: ReturnType<typeof setTimeout>;

	async function loadUsers() {
		const token = auth.getToken();
		if (!token) {
			error = "Not authenticated";
			loading = false;
			return;
		}

		loading = true;
		error = null;

		try {
			const result = await adminCommands.listUsers(fetch, token, {
				limit: PAGE_SIZE,
				offset,
				role: roleFilter || undefined,
				status: statusFilter || undefined,
				search: searchQuery || undefined,
			});
			users = result.data;
			total = result.total;
			hasMore = result.hasMore;
		} catch (err) {
			error = err instanceof Error ? err.message : "Failed to load users";
		}
		loading = false;
	}

	function handleSearch(e: Event) {
		const target = e.target as HTMLInputElement;
		clearTimeout(searchTimeout);
		searchTimeout = setTimeout(() => {
			searchQuery = target.value;
			offset = 0;
			loadUsers();
		}, 300);
	}

	function handleRoleChange(e: Event) {
		const target = e.target as HTMLSelectElement;
		roleFilter = target.value as UserRole | "";
		offset = 0;
		loadUsers();
	}

	function handleStatusChange(e: Event) {
		const target = e.target as HTMLSelectElement;
		statusFilter = target.value as UserStatus | "";
		offset = 0;
		loadUsers();
	}

	function prevPage() {
		if (offset > 0) {
			offset = Math.max(0, offset - PAGE_SIZE);
			loadUsers();
		}
	}

	function nextPage() {
		if (hasMore) {
			offset += PAGE_SIZE;
			loadUsers();
		}
	}

	function getRoleAccent(role: string): string {
		switch (role) {
			case "superadmin": return "#dc2626";
			case "admin": return "#8b5cf6";
			case "support": return "#3b82f6";
			case "editor": return "#14b8a6";
			case "tutor": return "#22c55e";
			case "tester": return "#f97316";
			default: return "#64748b";
		}
	}

	function getStatusAccent(status: string): string {
		switch (status) {
			case "active": return "#22c55e";
			case "suspended": return "#f97316";
			case "deleted": return "#dc2626";
			default: return "#64748b";
		}
	}

	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleDateString();
	}

	onMount(() => {
		loadUsers();
	});
</script>

<div class="users-page">
	<header class="users-page__header">
		<div class="users-page__title-row">
			<h1 class="users-page__title">Users</h1>
			<span class="users-page__count">{total} total</span>
		</div>
		<p class="users-page__subtitle">Manage user accounts and roles</p>
	</header>

	<div class="users-page__filters">
		<div class="users-page__search">
			<Search class="users-page__search-icon" />
			<input
				type="text"
				placeholder="Search by email..."
				class="users-page__search-input"
				oninput={handleSearch}
			/>
		</div>

		<select class="users-page__select" onchange={handleRoleChange}>
			<option value="">All roles</option>
			<option value={UserRoleConst.Student}>Student</option>
			<option value={UserRoleConst.Tester}>Tester</option>
			<option value={UserRoleConst.Tutor}>Tutor</option>
			<option value={UserRoleConst.Editor}>Editor</option>
			<option value={UserRoleConst.Admin}>Admin</option>
			<option value={UserRoleConst.Support}>Support</option>
			<option value={UserRoleConst.Superadmin}>Superadmin</option>
		</select>

		<select class="users-page__select" onchange={handleStatusChange}>
			<option value="">All statuses</option>
			<option value={UserStatusConst.Active}>Active</option>
			<option value={UserStatusConst.Suspended}>Suspended</option>
			<option value={UserStatusConst.Deleted}>Deleted</option>
		</select>
	</div>

	<Card variant="muted">
		{#if loading}
			<div class="users-page__loading">Loading...</div>
		{:else if error}
			<div class="users-page__error">{error}</div>
		{:else if users.length === 0}
			<div class="users-page__empty">No users found</div>
		{:else}
			<table class="users-table">
				<thead>
					<tr>
						<th>Email</th>
						<th>Display name</th>
						<th>Role</th>
						<th>Status</th>
						<th>Created</th>
					</tr>
				</thead>
				<tbody>
					{#each users as user}
						<tr onclick={() => goto(`/users/${user.id}`)} class="users-table__row--clickable">
							<td class="users-table__email">{user.email}</td>
							<td class="users-table__name">{user.displayName || "-"}</td>
							<td>
								<Pill accent={getRoleAccent(user.role)}>{user.role}</Pill>
							</td>
							<td>
								<Pill accent={getStatusAccent(user.status)}>{user.status}</Pill>
							</td>
							<td class="users-table__date">{formatDate(user.createdAt)}</td>
						</tr>
					{/each}
				</tbody>
			</table>

			<div class="users-page__pagination">
				<span class="users-page__pagination-info">
					Showing {offset + 1}-{Math.min(offset + users.length, total)} of {total}
				</span>
				<div class="users-page__pagination-controls">
					<button
						type="button"
						class="users-page__pagination-btn"
						disabled={offset === 0}
						onclick={prevPage}
					>
						<ChevronLeft />
					</button>
					<button
						type="button"
						class="users-page__pagination-btn"
						disabled={!hasMore}
						onclick={nextPage}
					>
						<ChevronRight />
					</button>
				</div>
			</div>
		{/if}
	</Card>
</div>

<style>
	.users-page__header {
		margin-bottom: 1.5rem;
	}

	.users-page__title-row {
		display: flex;
		align-items: baseline;
		gap: 0.75rem;
	}

	.users-page__title {
		margin: 0;
		font-size: 1.8rem;
		letter-spacing: -0.02em;
		font-weight: 650;
	}

	.users-page__count {
		color: var(--admin-color-text-muted);
		font-size: 0.95rem;
	}

	.users-page__subtitle {
		margin: 0.35rem 0 0;
		color: var(--admin-color-text-muted);
		font-size: 0.95rem;
	}

	.users-page__filters {
		display: flex;
		gap: 0.75rem;
		margin-bottom: 1rem;
	}

	.users-page__search {
		flex: 1;
		position: relative;
	}

	:global(.users-page__search-icon) {
		position: absolute;
		left: 0.75rem;
		top: 50%;
		transform: translateY(-50%);
		width: 1rem;
		height: 1rem;
		color: var(--admin-color-text-muted);
		pointer-events: none;
	}

	.users-page__search-input {
		width: 100%;
		padding: 0.6rem 0.75rem 0.6rem 2.25rem;
		background: var(--admin-color-surface-subtle);
		border: 1px solid var(--admin-color-border-subtle);
		border-radius: 0.5rem;
		color: inherit;
		font-size: 0.9rem;
	}

	.users-page__search-input:focus {
		outline: none;
		border-color: var(--admin-color-accent);
	}

	.users-page__select {
		padding: 0.6rem 0.75rem;
		background: var(--admin-color-surface-subtle);
		border: 1px solid var(--admin-color-border-subtle);
		border-radius: 0.5rem;
		color: inherit;
		font-size: 0.9rem;
		min-width: 140px;
	}

	.users-page__select:focus {
		outline: none;
		border-color: var(--admin-color-accent);
	}

	.users-page__loading,
	.users-page__error,
	.users-page__empty {
		padding: 2rem;
		text-align: center;
		color: var(--admin-color-text-muted);
	}

	.users-page__error {
		color: #fca5a5;
	}

	.users-table {
		width: 100%;
		border-collapse: collapse;
	}

	.users-table th {
		text-align: left;
		padding: 0.75rem 1rem;
		font-size: 0.8rem;
		font-weight: 500;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--admin-color-text-muted);
		border-bottom: 1px solid var(--admin-color-border-subtle);
	}

	.users-table td {
		padding: 0.75rem 1rem;
		border-bottom: 1px solid var(--admin-color-border-subtle);
	}

	.users-table tbody tr:last-child td {
		border-bottom: none;
	}

	.users-table__row--clickable {
		cursor: pointer;
		transition: background 0.15s ease;
	}

	.users-table__row--clickable:hover {
		background: rgba(148, 163, 184, 0.08);
	}

	.users-table__email {
		font-weight: 500;
	}

	.users-table__name {
		color: var(--admin-color-text-muted);
	}

	.users-table__date {
		color: var(--admin-color-text-muted);
		font-size: 0.9rem;
	}

	.users-page__pagination {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.75rem 1rem;
		border-top: 1px solid var(--admin-color-border-subtle);
	}

	.users-page__pagination-info {
		color: var(--admin-color-text-muted);
		font-size: 0.85rem;
	}

	.users-page__pagination-controls {
		display: flex;
		gap: 0.5rem;
	}

	.users-page__pagination-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 2rem;
		height: 2rem;
		padding: 0;
		background: transparent;
		border: 1px solid var(--admin-color-border-subtle);
		border-radius: 0.4rem;
		color: var(--admin-color-text);
		cursor: pointer;
	}

	.users-page__pagination-btn:hover:not(:disabled) {
		background: rgba(148, 163, 184, 0.16);
	}

	.users-page__pagination-btn:disabled {
		opacity: 0.3;
		cursor: not-allowed;
	}

	:global(.users-page__pagination-btn svg) {
		width: 1rem;
		height: 1rem;
	}

	@media (max-width: 768px) {
		.users-page__filters {
			flex-direction: column;
		}

		.users-table {
			font-size: 0.85rem;
		}

		.users-table th,
		.users-table td {
			padding: 0.5rem 0.75rem;
		}
	}
</style>
