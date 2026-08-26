<script lang="ts">
	import { AdminUserMenu as UnderlayAdminUserMenu } from "@inflatable-cookie/underlay/templates";
	import { auth } from "$lib/stores/auth";
	import type { LoginUser } from "@api-client";

	interface Props {
		currentUser?: LoginUser | null;
		variant?: "desktop" | "mobile";
		onNavigate?: () => void;
	}

	let {
		currentUser = null,
		variant = "desktop",
		onNavigate
	}: Props = $props();
</script>

<UnderlayAdminUserMenu
	displayName={currentUser?.displayName}
	roles={currentUser?.roles}
	appName="Acme Admin"
	avatarGradient="linear-gradient(135deg, #22c55e, #14b8a6)"
	onLogout={() => auth.logout()}
	{variant}
	{onNavigate}
/>
