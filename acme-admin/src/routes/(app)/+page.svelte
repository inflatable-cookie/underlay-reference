<script lang="ts">
	import { onMount } from "svelte";
	import { healthCommands, adminCommands, type DashboardStats, type ActivityEntry } from "@api-client";
	import { Card, Pill, StatCard, StatGrid } from "@decodelabs/underlay/components";
	import Users from "lucide-svelte/icons/users";
	import Image from "lucide-svelte/icons/image";
	import UserPlus from "lucide-svelte/icons/user-plus";
	import Activity from "lucide-svelte/icons/activity";
	import { auth } from "$lib/stores/auth";
	import ActivityFeed from "$lib/components/ActivityFeed.svelte";

	let healthStatus = $state<string | null>(null);
	let healthError = $state<string | null>(null);
	let checkedAt = $state<Date | null>(null);

	let stats = $state<DashboardStats | null>(null);
	let statsError = $state<string | null>(null);
	let statsLoading = $state(true);

	let recentActivity = $state<ActivityEntry[]>([]);
	let activityError = $state<string | null>(null);
	let activityLoading = $state(true);

	onMount(async () => {
		// Fetch health status
		try {
			const res = await healthCommands.health(fetch);
			healthStatus = res.status;
			checkedAt = new Date();
		} catch (err) {
			healthError = err instanceof Error ? err.message : "Failed to fetch health";
		}

		const token = auth.getToken();
		if (token) {
			// Fetch dashboard stats
			try {
				stats = await adminCommands.getDashboardStats(fetch, token);
			} catch (err) {
				statsError = err instanceof Error ? err.message : "Failed to fetch stats";
			}

			// Fetch recent activity
			try {
				const activityResponse = await adminCommands.listActivity(fetch, token, { limit: 10 });
				recentActivity = activityResponse.data;
			} catch (err) {
				activityError = err instanceof Error ? err.message : "Failed to fetch activity";
			}
		} else {
			statsError = "Not authenticated";
			activityError = "Not authenticated";
		}
		statsLoading = false;
		activityLoading = false;
	});
</script>

<div class="dashboard">
	<header class="dashboard__header">
		<h1 class="dashboard__title">Dashboard</h1>
		<p class="dashboard__subtitle">Platform overview and key metrics</p>
	</header>

	<StatGrid columns={2} minItemWidth={280}>
		<StatCard
			title="Users"
			value={stats?.userCounts.total ?? 0}
			label="Total users"
			variant="info"
			loading={statsLoading}
			error={statsError}
		>
			{#snippet icon()}<Users />{/snippet}
			{#snippet breakdown()}
				<span class="breakdown__item">
					<Pill accent="#22c55e">{stats?.userCounts.active ?? 0}</Pill>
					<span>active</span>
				</span>
				<span class="breakdown__item">
					<Pill accent="#f97316">{stats?.userCounts.suspended ?? 0}</Pill>
					<span>suspended</span>
				</span>
			{/snippet}
		</StatCard>

		<StatCard
			title="Media"
			value={stats?.mediaCount ?? 0}
			label="Media items"
			variant="warning"
			loading={statsLoading}
			error={statsError}
		>
			{#snippet icon()}<Image />{/snippet}
		</StatCard>

		<StatCard
			title="Recent registrations"
			value={stats?.recentRegistrations ?? 0}
			label="Last 7 days"
			variant="success"
			loading={statsLoading}
			error={statsError}
		>
			{#snippet icon()}<UserPlus />{/snippet}
		</StatCard>

		<StatCard
			title="Active sessions"
			value={stats?.activeSessions ?? 0}
			label="Logged in now"
			variant="danger"
			loading={statsLoading}
			error={statsError}
		>
			{#snippet icon()}<Activity />{/snippet}
		</StatCard>

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
	</StatGrid>

	<section class="dashboard__section">
		<h2 class="dashboard__section-title">Recent Activity</h2>
		<div class="dashboard__activity">
			<ActivityFeed
				activities={recentActivity}
				loading={activityLoading}
				error={activityError}
				emptyMessage="No recent activity"
			/>
		</div>
	</section>
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

	.breakdown__item {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		font-size: 0.85rem;
		color: var(--admin-color-text-muted);
	}

	.dashboard__section {
		margin-top: 2rem;
	}

	.dashboard__section-title {
		margin: 0 0 1rem;
		font-size: 1.1rem;
		font-weight: 600;
	}

	.dashboard__activity {
		background: var(--admin-color-surface);
		border: 1px solid var(--admin-color-border-subtle);
		border-radius: 0.5rem;
		padding: 1rem;
	}
</style>
