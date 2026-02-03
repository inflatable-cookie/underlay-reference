<script lang="ts">
	import { AuthLayout } from "@decodelabs/underlay/components";

	let { children } = $props();
</script>

<AuthLayout>
	{@render children()}
</AuthLayout>
