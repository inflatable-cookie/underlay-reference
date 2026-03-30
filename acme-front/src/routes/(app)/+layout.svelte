<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { auth, currentUser, authLoading } from "$lib/stores/auth";
  import { Button } from "@poodle/svelte-primitives";
  import { PageLoading } from "@poodle/svelte-composites";
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
  <PageLoading presentation="inline" message="Loading..." />
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
      <Button type="button" variant="ghost" on:click={handleLogout}>
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
    background: color-mix(in srgb, var(--poodle-color-background-panel) 96%, transparent);
    border-bottom: 1px solid color-mix(in srgb, var(--poodle-color-border-subtle) 82%, transparent);
  }

  .logo {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--poodle-color-accent-base);
    text-decoration: none;
  }

  .nav {
    flex: 1;
    display: flex;
    gap: 1.5rem;
  }

  .nav a {
    color: var(--poodle-color-text-secondary);
    text-decoration: none;
    font-size: 0.875rem;
  }

  .nav a:hover {
    color: var(--poodle-color-text-primary);
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
    color: var(--poodle-color-text-secondary);
  }

  .main {
    padding: 1.5rem 1.25rem;
  }
</style>
