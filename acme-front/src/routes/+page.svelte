<script lang="ts">
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";
	import { auth, currentUser } from "$lib/stores/auth";
	import { healthCommands } from "@api-client";
	import { Button } from "@decodelabs/underlay/components";

	let healthStatus = $state<string | null>(null);

	onMount(async () => {
		// Initialize auth
		await auth.initialize();

		// Check API health in background
		try {
			const res = await healthCommands.health(fetch);
			healthStatus = res.status;
		} catch {
			// Ignore health check errors on home page
		}
	});

	// Redirect authenticated users to dashboard
	$effect(() => {
		if ($currentUser) {
			goto("/dashboard");
		}
	});
</script>

<div class="hero">
	<h1>Acme Tasks</h1>
	<p class="tagline">A simple task management app built with Underlay patterns.</p>

	<div class="cta">
		<Button type="button" variant="primary" onclick={() => goto("/login")}>
			Sign In
		</Button>
		<Button type="button" variant="secondary" onclick={() => goto("/register")}>
			Create Account
		</Button>
	</div>

	{#if healthStatus}
		<p class="health">
			API Status: <span class="status-ok">{healthStatus}</span>
		</p>
	{/if}
</div>

<style>
	.hero {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		min-height: 60vh;
		text-align: center;
	}

	h1 {
		font-size: 2.5rem;
		letter-spacing: -0.02em;
		margin: 0 0 0.5rem;
		color: var(--text-primary, #0f172a);
	}

	.tagline {
		font-size: 1.125rem;
		color: var(--text-secondary, #6b7280);
		margin: 0 0 2rem;
	}

	.cta {
		display: flex;
		gap: 1rem;
	}

	.health {
		margin-top: 3rem;
		font-size: 0.875rem;
		color: var(--text-secondary, #9ca3af);
	}

	.status-ok {
		color: var(--success-color, #22c55e);
		font-weight: 500;
	}
</style>
