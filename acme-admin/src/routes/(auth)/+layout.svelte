<script lang="ts">
	import { Card } from "@poodle/svelte";

	let { children } = $props();
</script>

<div class="auth-layout">
	<div class="auth-layout__card">
		<Card variant="elevated">
			{@render children()}
		</Card>
	</div>
</div>

<style>
	.auth-layout {
		min-height: 100vh;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: clamp(1.5rem, 4vw, 3rem);
		box-sizing: border-box;
	}

	.auth-layout__card {
		width: 100%;
		max-width: 24rem;
	}
</style>
