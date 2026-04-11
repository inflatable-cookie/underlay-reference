<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { PageHeader as PoodlePageHeader } from "@poodle/svelte";
  import { Tabs, type TabItem } from "@poodle/svelte";

  let { children } = $props();

  // Map routes to tab values
  const routeToTab: Record<string, string> = {
    "/account": "overview",
    "/account/password": "password",
    "/account/2fa": "2fa",
    "/account/passkeys": "passkeys"
  };

  const tabToRoute: Record<string, string> = {
    "overview": "/account",
    "password": "/account/password",
    "2fa": "/account/2fa",
    "passkeys": "/account/passkeys"
  };

  // Get current tab from pathname
  const currentTab = $derived(routeToTab[$page.url.pathname] ?? "overview");
  const tabItems = $derived<TabItem[]>([
    { value: "overview", label: "Overview" },
    { value: "password", label: "Password" },
    { value: "2fa", label: "Two-Factor Auth" },
    { value: "passkeys", label: "Passkeys" }
  ]);

  // Local state for the tab component
  let activeTab = $state("overview");
  let lastSyncedPath = $state("");

  // Sync local state with URL changes (back/forward navigation)
  $effect(() => {
    if ($page.url.pathname !== lastSyncedPath) {
      activeTab = currentTab;
      lastSyncedPath = $page.url.pathname;
    }
  });

  // Navigate when user clicks a different tab
  $effect(() => {
    const route = tabToRoute[activeTab];
    if (route && route !== $page.url.pathname) {
      lastSyncedPath = route;
      goto(route);
    }
  });
</script>

<PoodlePageHeader title="Account" />

<Tabs
  value={activeTab}
  items={tabItems}
  variant="pill"
  ariaLabel="Account sections"
  on:valueChange={(event) => { activeTab = event.detail.value; }}
>
  {@render children?.()}
</Tabs>
