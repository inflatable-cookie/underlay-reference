<script lang="ts">
	import { onMount } from "svelte";
	import { adminCommands, type DashboardStats, type ActivityEntry } from "@api-client";
	import { Pill, StatCard, StatGrid } from "@decodelabs/underlay/components";
	import Users from "lucide-svelte/icons/users";
	import Image from "lucide-svelte/icons/image";
	import Settings from "lucide-svelte/icons/settings";
	import Box from "lucide-svelte/icons/box";
	import { auth } from "$lib/stores/auth";
	import LogList from "$lib/components/LogList.svelte";

	let stats = $state<DashboardStats | null>(null);
	let statsError = $state<string | null>(null);
	let statsLoading = $state(true);

	let recentActivity = $state<ActivityEntry[]>([]);
	let activityError = $state<string | null>(null);
	let activityLoading = $state(true);

	onMount(async () => {
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
			href="/users"
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
			href="/media"
		>
			{#snippet icon()}<Image />{/snippet}
		</StatCard>

		<div class="stat-card--acme">
			<StatCard
				title="Acme"
				value="–"
				label="Projects & Categories"
				loading={statsLoading}
				href="/projects"
			>
				{#snippet icon()}<Box />{/snippet}
			</StatCard>
		</div>

		<StatCard
			title="System"
			value={stats?.activeSessions ?? 0}
			label="Active sessions"
			loading={statsLoading}
			error={statsError}
			href="/system"
		>
			{#snippet icon()}<Settings />{/snippet}
		</StatCard>
	</StatGrid>

	<section class="dashboard__section">
		<h2 class="dashboard__section-title">Recent Activity</h2>
		<div class="dashboard__activity">
			<LogList
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

	/* Custom purple styling for Acme card to match nav badge */
	.stat-card--acme :global(.underlay-stat-card__icon) {
		background: color-mix(in srgb, #8b5cf6 15%, transparent);
		color: #8b5cf6;
	}
</style>
