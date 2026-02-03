<script lang="ts">
	import { onMount } from "svelte";
	import { healthCommands } from "@api-client";
	import { Card, Pill } from "@decodelabs/underlay/components";

	let healthStatus = $state<string | null>(null);
	let errorMessage = $state<string | null>(null);
	let checkedAt = $state<Date | null>(null);

	onMount(async () => {
		try {
			const res = await healthCommands.health(fetch);
			healthStatus = res.status;
			checkedAt = new Date();
		} catch (err) {
			errorMessage = err instanceof Error ? err.message : "Failed to fetch health";
		}
	});
</script>

<div class="dashboard">
	<header class="dashboard__header">
		<h1 class="dashboard__title">Dashboard</h1>
		<p class="dashboard__subtitle">Work in progress. This page is a smoke test for API wiring.</p>
	</header>

	<div class="dashboard__grid">
		<Card title="API health" variant="muted">
			{#if errorMessage}
				<p class="dashboard__error">{errorMessage}</p>
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

		<Card title="Next" variant="muted">
			<p class="dashboard__meta">Add admin sections: People, Businesses, Assessments, System.</p>
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

	@media (max-width: 900px) {
		.dashboard__grid {
			grid-template-columns: 1fr;
		}
	}
</style>
