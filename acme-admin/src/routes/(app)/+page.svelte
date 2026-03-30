<script lang="ts">
	import { onMount } from "svelte";
	import { adminCommands, type DashboardStats, type ActivityEntry } from "@api-client";
	import { LogList, MetricTile, type LogEntry } from "@poodle/svelte-composites";
	import { Pill } from "@poodle/svelte-primitives";
	import Users from "lucide-svelte/icons/users";
	import Image from "lucide-svelte/icons/image";
	import Settings from "lucide-svelte/icons/settings";
	import Box from "lucide-svelte/icons/box";
	import { auth } from "$lib/stores/auth";

	let stats = $state<DashboardStats | null>(null);
	let statsError = $state<string | null>(null);
	let statsLoading = $state(true);

	let recentActivity = $state<ActivityEntry[]>([]);
	let activityError = $state<string | null>(null);
	let activityLoading = $state(true);
	const logEntries = $derived<LogEntry[]>(
		recentActivity.map((activity) => ({
			id: activity.id,
			occurredAt: activity.occurredAt,
			actor: activity.actor
				? {
						id: activity.actor.id,
						email: activity.actor.email,
						name: activity.actor.displayName ?? undefined
				  }
				: undefined,
			action: activity.action,
			resourceType: activity.resourceType,
			resourceId: activity.resourceId,
			resourceLabel: (activity.details?.resourceLabel as string | undefined) ?? undefined,
			details: activity.details
		}))
	);

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

	<div class="dashboard__metrics">
		<a class="dashboard__metric-link" href="/users">
			<div class="dashboard__metric-header">
				<div class="dashboard__metric-heading">
					<Users />
					<span>Users</span>
				</div>
				{#if statsError}
					<Pill tone="danger" appearance="subtle">Sync issue</Pill>
				{:else if statsLoading}
					<Pill appearance="subtle">Loading</Pill>
				{/if}
			</div>
			<MetricTile
				label="Total users"
				value={statsLoading ? "..." : String(stats?.userCounts.total ?? 0)}
				trend="up"
				trendLabel={`${stats?.userCounts.active ?? 0} active`}
				ariaLabel="Users metric"
			/>
			<div class="dashboard__metric-footer">
				<span class="breakdown__item">
					<Pill tone="success">{stats?.userCounts.active ?? 0}</Pill>
					<span>active</span>
				</span>
				<span class="breakdown__item">
					<Pill tone="danger" appearance="subtle">{stats?.userCounts.suspended ?? 0}</Pill>
					<span>suspended</span>
				</span>
			</div>
		</a>

		<a class="dashboard__metric-link" href="/media">
			<div class="dashboard__metric-header">
				<div class="dashboard__metric-heading">
					<Image />
					<span>Media</span>
				</div>
			</div>
			<MetricTile
				label="Media items"
				value={statsLoading ? "..." : String(stats?.mediaCount ?? 0)}
				trend="flat"
				trendLabel={statsError ? "Awaiting refresh" : "Library inventory"}
				ariaLabel="Media metric"
			/>
			<p class="dashboard__metric-copy">Asset volume under the active catalog and ingestion flow.</p>
		</a>

		<a class="dashboard__metric-link" href="/projects">
			<div class="dashboard__metric-header">
				<div class="dashboard__metric-heading">
					<Box />
					<span>Acme</span>
				</div>
			</div>
			<MetricTile
				label="Recent registrations"
				value={statsLoading ? "..." : String(stats?.recentRegistrations ?? 0)}
				trend="up"
				trendLabel="Projects and categories"
				ariaLabel="Acme metric"
			/>
			<p class="dashboard__metric-copy">Project-side growth and taxonomy activity across the workspace.</p>
		</a>

		<a class="dashboard__metric-link" href="/system">
			<div class="dashboard__metric-header">
				<div class="dashboard__metric-heading">
					<Settings />
					<span>System</span>
				</div>
			</div>
			<MetricTile
				label="Active sessions"
				value={statsLoading ? "..." : String(stats?.activeSessions ?? 0)}
				trend="flat"
				trendLabel={statsError ? "Check auth state" : "Current operator load"}
				ariaLabel="System metric"
			/>
			<p class="dashboard__metric-copy">Operator activity and runtime posture for the current environment.</p>
		</a>
	</div>

	<section class="dashboard__section">
		<h2 class="dashboard__section-title">Recent Activity</h2>
		<LogList
			entries={logEntries}
			loading={activityLoading}
			error={activityError}
			emptyMessage="No recent activity"
		/>
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

	.dashboard__metrics {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
		gap: 1rem;
	}

	.dashboard__metric-link {
		display: grid;
		gap: 0.85rem;
		padding: 1rem;
		border: 1px solid var(--admin-color-border-subtle);
		border-radius: 0.9rem;
		background:
			linear-gradient(180deg, color-mix(in srgb, var(--admin-color-surface-card) 92%, transparent), var(--admin-color-surface)),
			var(--admin-color-surface-card);
		color: inherit;
		text-decoration: none;
		box-shadow: 0 18px 48px rgba(0, 0, 0, 0.18);
		transition:
			transform 160ms ease,
			border-color 160ms ease,
			box-shadow 160ms ease;
	}

	.dashboard__metric-link:hover {
		transform: translateY(-2px);
		border-color: var(--admin-color-border-strong);
		box-shadow: 0 24px 56px rgba(0, 0, 0, 0.28);
	}

	.dashboard__metric-link:focus-visible {
		outline: 2px solid var(--admin-color-accent);
		outline-offset: 3px;
	}

	.dashboard__metric-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.75rem;
	}

	.dashboard__metric-heading {
		display: inline-flex;
		align-items: center;
		gap: 0.55rem;
		font-size: 0.9rem;
		font-weight: 650;
		letter-spacing: 0.01em;
	}

	.dashboard__metric-heading :global(svg) {
		width: 1rem;
		height: 1rem;
		color: var(--admin-color-accent);
	}

	.dashboard__metric-footer {
		display: flex;
		flex-wrap: wrap;
		gap: 0.75rem;
	}

	.dashboard__metric-copy {
		margin: 0;
		color: var(--admin-color-text-muted);
		font-size: 0.85rem;
		line-height: 1.45;
	}

	.breakdown__item {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		font-size: 0.85rem;
		color: var(--admin-color-text-muted);
	}

	.dashboard__metric-link :global(.state-tile) {
		padding: 0;
		border: none;
		border-radius: 0;
		background: transparent;
	}

	.dashboard__metric-link :global(.state-tile__value) {
		font-size: 1.65rem;
	}

	.dashboard__section {
		margin-top: 2rem;
	}

	.dashboard__section-title {
		margin: 0 0 1rem;
		font-size: 1.1rem;
		font-weight: 600;
	}

</style>
