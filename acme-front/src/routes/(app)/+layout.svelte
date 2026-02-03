<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { auth, currentUser, authLoading } from "$lib/stores/auth";
  import { Button, PageLoading } from "@decodelabs/underlay/components";
  import LogOut from "lucide-svelte/icons/log-out";
  import User from "lucide-svelte/icons/user";

  let { children } = $props();

  onMount(async () => {
    await auth.initialize();
  });

  // Redirect to login if not authenticated
  $effect(() => {
    if (!$authLoading && !$currentUser) {
      goto("/login");
    }
  });

  async function handleLogout() {
    await auth.logout();
  }
</script>

{#if $authLoading}
  <PageLoading message="Loading..." />
{:else if $currentUser}
  <header class="header">
    <a href="/dashboard" class="logo">Acme</a>
    <nav class="nav">
      <a href="/dashboard">My Projects</a>
    </nav>
    <div class="user-menu">
      <span class="user-name">
        <User size={16} />
        {$currentUser.displayName}
      </span>
      <Button type="button" variant="subtle" onclick={handleLogout}>
        <LogOut size={16} />
        Sign Out
      </Button>
    </div>
  </header>
  <main class="main">
    {@render children()}
  </main>
{/if}

<style>
  .header {
    display: flex;
    align-items: center;
    gap: 2rem;
    padding: 0.75rem 1.25rem;
    background: white;
    border-bottom: 1px solid var(--border-color, #e5e7eb);
  }

  .logo {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--primary-color, #2563eb);
    text-decoration: none;
  }

  .nav {
    flex: 1;
    display: flex;
    gap: 1.5rem;
  }

  .nav a {
    color: var(--text-secondary, #6b7280);
    text-decoration: none;
    font-size: 0.875rem;
  }

  .nav a:hover {
    color: var(--text-primary, #111827);
  }

  .user-menu {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .user-name {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.875rem;
    color: var(--text-secondary, #6b7280);
  }

  .main {
    padding: 1.5rem 1.25rem;
  }
</style>
