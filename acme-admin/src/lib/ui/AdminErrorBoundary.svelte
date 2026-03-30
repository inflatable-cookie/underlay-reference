<script lang="ts">
	import type { Snippet } from "svelte";
	import { EmptyState } from "@poodle/svelte-composites";
	import { Button } from "@poodle/svelte-primitives";

	interface Props {
		children: Snippet;
	}

	let { children }: Props = $props();

	let currentError = $state<Error | null>(null);

	function handleError(error: unknown) {
		currentError = error instanceof Error ? error : new Error(String(error));
	}

	function reset() {
		currentError = null;
	}
</script>

{#if currentError}
	<EmptyState title="Something went wrong" message={currentError.message}>
		<Button slot="actions" variant="secondary" onclick={reset}>Try again</Button>
	</EmptyState>
{:else}
	<svelte:boundary onerror={handleError}>
		{@render children()}
	</svelte:boundary>
{/if}
