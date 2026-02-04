<script lang="ts">
	import { onMount } from "svelte";
	import { adminCommands, type UserDetail, type UserRole, UserRole as UserRoleConst, type ActivityEntry, type Session } from "@api-client";
	import { Card, Pill, Button, AlertDialog } from "@decodelabs/underlay/components";
	import { useToasts } from "@decodelabs/underlay/patterns";
	import ArrowLeft from "lucide-svelte/icons/arrow-left";
	import Shield from "lucide-svelte/icons/shield";
	import Ban from "lucide-svelte/icons/ban";
	import CheckCircle from "lucide-svelte/icons/check-circle";
	import X from "lucide-svelte/icons/x";
	import { auth } from "$lib/stores/auth";
	import ActivityFeed from "$lib/components/ActivityFeed.svelte";

	interface Props {
		data: { userId: string };
	}

	let { data }: Props = $props();
	const toastStore = useToasts();

	let user = $state<UserDetail | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let actionLoading = $state(false);

	let userActivity = $state<ActivityEntry[]>([]);
	let activityLoading = $state(true);
	let activityError = $state<string | null>(null);

	let userSessions = $state<Session[]>([]);
	let sessionsLoading = $state(true);
	let sessionsError = $state<string | null>(null);
	let sessionToRevoke = $state<Session | null>(null);
	let revokingSession = $state(false);
	let showRevokeDialog = $state(false);

	// Keep showRevokeDialog in sync with sessionToRevoke
	$effect(() => {
		showRevokeDialog = sessionToRevoke !== null;
	});

	let showRoleModal = $state(false);
	let selectedRole = $state<UserRole>("user");

	async function loadUser() {
		const token = auth.getToken();
		if (!token) {
			error = "Not authenticated";
			loading = false;
			activityLoading = false;
			sessionsLoading = false;
			return;
		}

		loading = true;
		error = null;

		try {
			user = await adminCommands.getUser(data.userId, fetch, token);
			selectedRole = user.role;
		} catch (err) {
			error = err instanceof Error ? err.message : "Failed to load user";
		}
		loading = false;

		// Load user sessions
		sessionsLoading = true;
		sessionsError = null;
		try {
			userSessions = await adminCommands.listUserSessions(data.userId, fetch, token);
		} catch (err) {
			sessionsError = err instanceof Error ? err.message : "Failed to load sessions";
		}
		sessionsLoading = false;

		// Load user activity
		activityLoading = true;
		activityError = null;
		try {
			const activityResponse = await adminCommands.listActivityForUser(data.userId, fetch, token, { limit: 10 });
			userActivity = activityResponse.data;
		} catch (err) {
			activityError = err instanceof Error ? err.message : "Failed to load activity";
		}
		activityLoading = false;
	}

	async function handleSuspend() {
		if (!user) return;

		const token = auth.getToken();
		if (!token) return;

		actionLoading = true;
		try {
			await adminCommands.suspendUser(user.id, fetch, token);
			toastStore.push({ variant: "success", message: "User suspended" });
			await loadUser();
		} catch (err) {
			toastStore.push({ variant: "error", message: err instanceof Error ? err.message : "Failed to suspend user" });
		}
		actionLoading = false;
	}

	async function handleUnsuspend() {
		if (!user) return;

		const token = auth.getToken();
		if (!token) return;

		actionLoading = true;
		try {
			await adminCommands.unsuspendUser(user.id, fetch, token);
			toastStore.push({ variant: "success", message: "User reactivated" });
			await loadUser();
		} catch (err) {
			toastStore.push({ variant: "error", message: err instanceof Error ? err.message : "Failed to reactivate user" });
		}
		actionLoading = false;
	}

	async function handleRoleChange() {
		if (!user) return;

		const token = auth.getToken();
		if (!token) return;

		actionLoading = true;
		try {
			await adminCommands.updateUserRole(user.id, { role: selectedRole }, fetch, token);
			toastStore.push({ variant: "success", message: "Role updated" });
			showRoleModal = false;
			await loadUser();
		} catch (err) {
			toastStore.push({ variant: "error", message: err instanceof Error ? err.message : "Failed to update role" });
		}
		actionLoading = false;
	}

	async function handleRevokeSession() {
		if (!sessionToRevoke) return;

		const token = auth.getToken();
		if (!token) return;

		revokingSession = true;
		try {
			await adminCommands.revokeUserSession(data.userId, sessionToRevoke.id, fetch, token);
			toastStore.push({ variant: "success", message: "Session revoked" });
			sessionToRevoke = null;
			// Reload sessions
			userSessions = await adminCommands.listUserSessions(data.userId, fetch, token);
		} catch (err) {
			toastStore.push({ variant: "error", message: err instanceof Error ? err.message : "Failed to revoke session" });
		}
		revokingSession = false;
	}

	function getSessionStatusAccent(status: string): string {
		switch (status) {
			case "active": return "#22c55e";
			case "expired": return "#f59e0b";
			case "revoked": return "#dc2626";
			default: return "#64748b";
		}
	}

	function truncateUserAgent(ua: string | null | undefined, max = 50): string {
		if (!ua) return "-";
		return ua.length > max ? ua.substring(0, max) + "..." : ua;
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
		return new Date(dateStr).toLocaleString();
	}

	onMount(() => {
		loadUser();
	});
</script>

<div class="user-detail">
	<a href="/users" class="user-detail__back">
		<ArrowLeft />
		<span>Back to users</span>
	</a>

	{#if loading}
		<Card variant="muted">
			<div class="user-detail__loading">Loading...</div>
		</Card>
	{:else if error}
		<Card variant="muted">
			<div class="user-detail__error">{error}</div>
		</Card>
	{:else if user}
		<header class="user-detail__header">
			<div>
				<h1 class="user-detail__title">{user.email}</h1>
				{#if user.displayName}
					<p class="user-detail__subtitle">{user.displayName}</p>
				{/if}
			</div>
			<div class="user-detail__badges">
				<Pill accent={getRoleAccent(user.role)}>{user.role}</Pill>
				<Pill accent={getStatusAccent(user.status)}>{user.status}</Pill>
			</div>
		</header>

		<div class="user-detail__grid">
			<Card title="User info" variant="muted">
				<dl class="user-detail__info">
					<div class="user-detail__info-row">
						<dt>User ID</dt>
						<dd class="user-detail__mono">{user.id}</dd>
					</div>
					<div class="user-detail__info-row">
						<dt>Email</dt>
						<dd>{user.email}</dd>
					</div>
					<div class="user-detail__info-row">
						<dt>Display name</dt>
						<dd>{user.displayName || "-"}</dd>
					</div>
					<div class="user-detail__info-row">
						<dt>Created</dt>
						<dd>{formatDate(user.createdAt)}</dd>
					</div>
					<div class="user-detail__info-row">
						<dt>Last updated</dt>
						<dd>{formatDate(user.updatedAt)}</dd>
					</div>
				</dl>
			</Card>

			<Card title="Security" variant="muted">
				<dl class="user-detail__info">
					<div class="user-detail__info-row">
						<dt>Active sessions</dt>
						<dd>{user.activeSessionCount}</dd>
					</div>
					<div class="user-detail__info-row">
						<dt>Failed login attempts</dt>
						<dd>{user.failedLoginCount}</dd>
					</div>
					<div class="user-detail__info-row">
						<dt>Lockout until</dt>
						<dd>{user.lockoutUntil ? formatDate(user.lockoutUntil) : "-"}</dd>
					</div>
				</dl>

				<h4 class="user-detail__section-title">Sessions</h4>
				{#if sessionsLoading}
					<p class="user-detail__muted">Loading sessions...</p>
				{:else if sessionsError}
					<p class="user-detail__error-text">{sessionsError}</p>
				{:else if userSessions.length === 0}
					<p class="user-detail__muted">No sessions found</p>
				{:else}
					<div class="sessions-table-wrapper">
						<table class="sessions-table">
							<thead>
								<tr>
									<th>Status</th>
									<th>IP Address</th>
									<th>User Agent</th>
									<th>Created</th>
									<th>Last Used</th>
									<th></th>
								</tr>
							</thead>
							<tbody>
								{#each userSessions as session}
									<tr class:revoked={session.status === "revoked"} class:expired={session.status === "expired"}>
										<td>
											<Pill accent={getSessionStatusAccent(session.status)}>{session.status}</Pill>
										</td>
										<td class="sessions-table__mono">{session.ipAddress || "-"}</td>
										<td title={session.userAgent || undefined}>{truncateUserAgent(session.userAgent)}</td>
										<td>{formatDate(session.createdAt)}</td>
										<td>{formatDate(session.lastUsedAt)}</td>
										<td>
											{#if session.status === "active"}
												<Button
													variant="danger"
													size="sm"
													onclick={() => (sessionToRevoke = session)}
												>
													<X />
													<span>Revoke</span>
												</Button>
											{:else if session.revocationReason}
												<span class="sessions-table__reason" title={session.revocationReason}>
													{session.revocationReason}
												</span>
											{/if}
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			</Card>
		</div>

		<Card title="Actions" variant="muted">
			<div class="user-detail__actions">
				<Button variant="secondary" onclick={() => (showRoleModal = true)} disabled={actionLoading}>
					<Shield />
					<span>Change role</span>
				</Button>

				{#if user.status === "active"}
					<Button variant="danger" onclick={handleSuspend} disabled={actionLoading}>
						<Ban />
						<span>Suspend user</span>
					</Button>
				{:else if user.status === "suspended"}
					<Button variant="secondary" onclick={handleUnsuspend} disabled={actionLoading}>
						<CheckCircle />
						<span>Reactivate user</span>
					</Button>
				{/if}
			</div>
		</Card>

		<Card title="User Activity" variant="muted">
			<ActivityFeed
				activities={userActivity}
				loading={activityLoading}
				error={activityError}
				emptyMessage="No activity recorded for this user"
			/>
		</Card>
	{/if}
</div>

{#if showRoleModal}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="modal-backdrop" onclick={() => (showRoleModal = false)} role="presentation">
		<!-- svelte-ignore a11y_interactive_supports_focus -->
		<div class="modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
			<h2 class="modal__title">Change user role</h2>
			<p class="modal__subtitle">Select a new role for this user.</p>

			<select
				class="modal__select"
				bind:value={selectedRole}
			>
				<option value={UserRoleConst.User}>User</option>
				<option value={UserRoleConst.Tester}>Tester</option>
				<option value={UserRoleConst.Editor}>Editor</option>
				<option value={UserRoleConst.Admin}>Admin</option>
				<option value={UserRoleConst.Support}>Support</option>
				<option value={UserRoleConst.Superadmin}>Superadmin</option>
			</select>

			<div class="modal__actions">
				<Button variant="secondary" onclick={() => (showRoleModal = false)}>Cancel</Button>
				<Button variant="primary" onclick={handleRoleChange} disabled={actionLoading}>
					Save
				</Button>
			</div>
		</div>
	</div>
{/if}

<AlertDialog
	bind:open={showRevokeDialog}
	title="Revoke session"
	description="This will immediately log the user out of this session. They will need to log in again."
	confirmLabel="Revoke"
	showTrigger={false}
	onConfirm={handleRevokeSession}
	onCancel={() => (sessionToRevoke = null)}
/>

<style>
	.user-detail__back {
		display: inline-flex;
		align-items: center;
		gap: 0.4rem;
		color: var(--admin-color-text-muted);
		text-decoration: none;
		font-size: 0.9rem;
		margin-bottom: 1rem;
		transition: color 0.15s ease;
	}

	.user-detail__back:hover {
		color: var(--admin-color-text);
	}

	:global(.user-detail__back svg) {
		width: 1rem;
		height: 1rem;
	}

	.user-detail__loading,
	.user-detail__error {
		padding: 2rem;
		text-align: center;
		color: var(--admin-color-text-muted);
	}

	.user-detail__error {
		color: #fca5a5;
	}

	.user-detail__header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: 1rem;
		margin-bottom: 1.5rem;
	}

	.user-detail__title {
		margin: 0;
		font-size: 1.8rem;
		letter-spacing: -0.02em;
		font-weight: 650;
	}

	.user-detail__subtitle {
		margin: 0.35rem 0 0;
		color: var(--admin-color-text-muted);
		font-size: 0.95rem;
	}

	.user-detail__badges {
		display: flex;
		gap: 0.5rem;
	}

	.user-detail__grid {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 1rem;
		margin-bottom: 1rem;
	}

	.user-detail__info {
		margin: 0;
	}

	.user-detail__info-row {
		display: flex;
		justify-content: space-between;
		padding: 0.6rem 0;
		border-bottom: 1px solid var(--admin-color-border-subtle);
	}

	.user-detail__info-row:last-child {
		border-bottom: none;
	}

	.user-detail__info-row dt {
		color: var(--admin-color-text-muted);
		font-size: 0.9rem;
	}

	.user-detail__info-row dd {
		margin: 0;
		text-align: right;
		font-size: 0.9rem;
	}

	.user-detail__mono {
		font-family: monospace;
		font-size: 0.85rem;
	}

	.user-detail__section-title {
		margin: 1.5rem 0 0.75rem;
		font-size: 0.95rem;
		font-weight: 600;
		color: var(--admin-color-text);
	}

	.user-detail__muted {
		color: var(--admin-color-text-muted);
		font-size: 0.9rem;
		margin: 0;
	}

	.user-detail__error-text {
		color: #fca5a5;
		font-size: 0.9rem;
		margin: 0;
	}

	.sessions-table-wrapper {
		overflow-x: auto;
		margin-top: 0.5rem;
	}

	.sessions-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.85rem;
	}

	.sessions-table th {
		text-align: left;
		padding: 0.6rem 0.75rem;
		border-bottom: 1px solid var(--admin-color-border-subtle);
		color: var(--admin-color-text-muted);
		font-weight: 500;
		white-space: nowrap;
	}

	.sessions-table td {
		padding: 0.6rem 0.75rem;
		border-bottom: 1px solid var(--admin-color-border-subtle);
		vertical-align: middle;
	}

	.sessions-table tr:last-child td {
		border-bottom: none;
	}

	.sessions-table tr.revoked,
	.sessions-table tr.expired {
		opacity: 0.6;
	}

	.sessions-table__mono {
		font-family: monospace;
		font-size: 0.8rem;
	}

	.sessions-table__reason {
		color: var(--admin-color-text-muted);
		font-size: 0.8rem;
		font-style: italic;
	}

	:global(.sessions-table button svg) {
		width: 0.875rem;
		height: 0.875rem;
	}

	.user-detail__actions {
		display: flex;
		gap: 0.75rem;
	}

	:global(.user-detail__actions button svg) {
		width: 1rem;
		height: 1rem;
	}

	.modal-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 200;
	}

	.modal {
		background: var(--admin-color-surface);
		border: 1px solid var(--admin-color-border-subtle);
		border-radius: 0.75rem;
		padding: 1.5rem;
		width: 100%;
		max-width: 400px;
		box-shadow: 0 25px 50px rgba(0, 0, 0, 0.5);
	}

	.modal__title {
		margin: 0;
		font-size: 1.25rem;
		font-weight: 600;
	}

	.modal__subtitle {
		margin: 0.5rem 0 1.25rem;
		color: var(--admin-color-text-muted);
		font-size: 0.9rem;
	}

	.modal__select {
		width: 100%;
		padding: 0.6rem 0.75rem;
		background: var(--admin-color-surface-subtle);
		border: 1px solid var(--admin-color-border-subtle);
		border-radius: 0.5rem;
		color: inherit;
		font-size: 0.95rem;
		margin-bottom: 1.25rem;
	}

	.modal__select:focus {
		outline: none;
		border-color: var(--admin-color-accent);
	}

	.modal__actions {
		display: flex;
		justify-content: flex-end;
		gap: 0.75rem;
	}

	@media (max-width: 768px) {
		.user-detail__header {
			flex-direction: column;
		}

		.user-detail__grid {
			grid-template-columns: 1fr;
		}

		.user-detail__actions {
			flex-direction: column;
		}
	}
</style>
