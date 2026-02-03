<script lang="ts">
	import { onMount } from "svelte";
	import { healthCommands, adminCommands, type DashboardStats } from "@api-client";
	import { Card, Pill } from "@decodelabs/underlay/components";
	import Users from "lucide-svelte/icons/users";
	import Image from "lucide-svelte/icons/image";
	import UserPlus from "lucide-svelte/icons/user-plus";
	import Activity from "lucide-svelte/icons/activity";
	import { auth } from "$lib/stores/auth";

	let healthStatus = $state<string | null>(null);
	let healthError = $state<string | null>(null);
	let checkedAt = $state<Date | null>(null);

	let stats = $state<DashboardStats | null>(null);
	let statsError = $state<string | null>(null);
	let statsLoading = $state(true);

	onMount(async () => {
		// Fetch health status
		try {
			const res = await healthCommands.health(fetch);
			healthStatus = res.status;
			checkedAt = new Date();
		} catch (err) {
			healthError = err instanceof Error ? err.message : "Failed to fetch health";
		}

		// Fetch dashboard stats
		const token = auth.getToken();
		if (token) {
			try {
				stats = await adminCommands.getDashboardStats(fetch, token);
			} catch (err) {
				statsError = err instanceof Error ? err.message : "Failed to fetch stats";
			}
		} else {
			statsError = "Not authenticated";
		}
		statsLoading = false;
	});
</script>

<div class="dashboard">
	<header class="dashboard__header">
		<h1 class="dashboard__title">Dashboard</h1>
		<p class="dashboard__subtitle">Platform overview and key metrics</p>
	</header>

	<div class="dashboard__grid">
		<Card title="Users" variant="muted">
			{#if statsLoading}
				<p class="dashboard__meta">Loading...</p>
			{:else if statsError}
				<p class="dashboard__error">{statsError}</p>
			{:else if stats}
				<div class="stat-card">
					<div class="stat-card__icon stat-card__icon--users">
						<Users />
					</div>
					<div class="stat-card__content">
						<p class="stat-card__value">{stats.userCounts.total}</p>
						<p class="stat-card__label">Total users</p>
					</div>
				</div>
				<div class="stat-card__breakdown">
					<span class="stat-card__item">
						<Pill accent="#22c55e">{stats.userCounts.active}</Pill>
						<span>active</span>
					</span>
					<span class="stat-card__item">
						<Pill accent="#f97316">{stats.userCounts.suspended}</Pill>
						<span>suspended</span>
					</span>
				</div>
			{/if}
		</Card>

		<Card title="Media" variant="muted">
			{#if statsLoading}
				<p class="dashboard__meta">Loading...</p>
			{:else if statsError}
				<p class="dashboard__error">{statsError}</p>
			{:else if stats}
				<div class="stat-card">
					<div class="stat-card__icon stat-card__icon--media">
						<Image />
					</div>
					<div class="stat-card__content">
						<p class="stat-card__value">{stats.mediaCount}</p>
						<p class="stat-card__label">Media items</p>
					</div>
				</div>
			{/if}
		</Card>

		<Card title="Recent registrations" variant="muted">
			{#if statsLoading}
				<p class="dashboard__meta">Loading...</p>
			{:else if statsError}
				<p class="dashboard__error">{statsError}</p>
			{:else if stats}
				<div class="stat-card">
					<div class="stat-card__icon stat-card__icon--registrations">
						<UserPlus />
					</div>
					<div class="stat-card__content">
						<p class="stat-card__value">{stats.recentRegistrations}</p>
						<p class="stat-card__label">Last 7 days</p>
					</div>
				</div>
			{/if}
		</Card>

		<Card title="Active sessions" variant="muted">
			{#if statsLoading}
				<p class="dashboard__meta">Loading...</p>
			{:else if statsError}
				<p class="dashboard__error">{statsError}</p>
			{:else if stats}
				<div class="stat-card">
					<div class="stat-card__icon stat-card__icon--sessions">
						<Activity />
					</div>
					<div class="stat-card__content">
						<p class="stat-card__value">{stats.activeSessions}</p>
						<p class="stat-card__label">Logged in now</p>
					</div>
				</div>
			{/if}
		</Card>

		<Card title="API health" variant="muted">
			{#if healthError}
				<p class="dashboard__error">{healthError}</p>
			{:else if healthStatus}
				<p class="dashboard__row">
					<Pill accent={healthStatus === "ok" ? "#22c55e" : "#f97316"}>{healthStatus}</Pill>
					<span class="dashboard__meta">
						{checkedAt ? checkedAt.toLocaleTimeString() : ""}
					</span>
				</p>
			{:else}
				<p class="dashboard__meta">Checking...</p>
			{/if}
		</Card>
	</div>
</div>

<style>
	.dashboard__header {
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
		margin-bottom: 1rem;
	}

	.dashboard__title {
		margin: 0;
		font-size: 1.8rem;
		letter-spacing: -0.02em;
		font-weight: 650;
	}

	.dashboard__subtitle {
		margin: 0;
		color: var(--admin-color-text-muted);
		font-size: 0.95rem;
	}

	.dashboard__grid {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 1rem;
	}

	.dashboard__row {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		margin: 0;
	}

	.dashboard__meta {
		color: var(--admin-color-text-muted);
		opacity: 0.8;
		font-size: 0.9rem;
	}

	.dashboard__error {
		margin: 0;
		color: #fca5a5;
	}

	.stat-card {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.stat-card__icon {
		width: 3rem;
		height: 3rem;
		border-radius: 0.75rem;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.stat-card__icon--users {
		background: linear-gradient(135deg, #3b82f6, #8b5cf6);
	}

	.stat-card__icon--media {
		background: linear-gradient(135deg, #f97316, #eab308);
	}

	.stat-card__icon--registrations {
		background: linear-gradient(135deg, #22c55e, #14b8a6);
	}

	.stat-card__icon--sessions {
		background: linear-gradient(135deg, #ec4899, #f43f5e);
	}

	:global(.stat-card__icon svg) {
		width: 1.5rem;
		height: 1.5rem;
		color: white;
	}

	.stat-card__content {
		display: flex;
		flex-direction: column;
		gap: 0.1rem;
	}

	.stat-card__value {
		margin: 0;
		font-size: 1.75rem;
		font-weight: 650;
		line-height: 1;
	}

	.stat-card__label {
		margin: 0;
		color: var(--admin-color-text-muted);
		font-size: 0.85rem;
	}

	.stat-card__breakdown {
		display: flex;
		gap: 1rem;
		margin-top: 0.75rem;
		padding-top: 0.75rem;
		border-top: 1px solid var(--admin-color-border-subtle);
	}

	.stat-card__item {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		font-size: 0.85rem;
		color: var(--admin-color-text-muted);
	}

	@media (max-width: 900px) {
		.dashboard__grid {
			grid-template-columns: 1fr;
		}
	}
</style>
