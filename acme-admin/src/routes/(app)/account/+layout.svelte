<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { PageHeader } from "@decodelabs/underlay/patterns";
  import { TabsRoot, TabsList, TabsTrigger, TabsContent } from "@decodelabs/underlay/components";

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

<PageHeader section="Account" />

<TabsRoot bind:value={activeTab} variant="boxed" size="sm">
  <TabsList>
    <TabsTrigger value="overview">Overview</TabsTrigger>
    <TabsTrigger value="password">Password</TabsTrigger>
    <TabsTrigger value="2fa">Two-Factor Auth</TabsTrigger>
    <TabsTrigger value="passkeys">Passkeys</TabsTrigger>
  </TabsList>

  <TabsContent value={activeTab}>
    {@render children?.()}
  </TabsContent>
</TabsRoot>
