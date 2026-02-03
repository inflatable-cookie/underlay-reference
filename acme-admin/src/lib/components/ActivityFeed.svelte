<script lang="ts">
  import type { ActivityEntry } from "acme-client";
  import { Badge } from "@decodelabs/underlay/components";
  import { formatRelative } from "@decodelabs/underlay/patterns";
  import User from "lucide-svelte/icons/user";
  import Plus from "lucide-svelte/icons/plus";
  import Pencil from "lucide-svelte/icons/pencil";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import Upload from "lucide-svelte/icons/upload";
  import LogIn from "lucide-svelte/icons/log-in";
  import LogOut from "lucide-svelte/icons/log-out";
  import Shield from "lucide-svelte/icons/shield";

  interface Props {
    activities: ActivityEntry[];
    loading?: boolean;
    error?: string | null;
    emptyMessage?: string;
  }

  let { activities, loading = false, error = null, emptyMessage = "No activity yet" }: Props = $props();

  type ActionType = "create" | "update" | "delete" | "restore" | "upload" | "login" | "logout" | "security" | "other";

  function getActionType(action: string): ActionType {
    switch (action) {
      case "create":
      case "created":
        return "create";
      case "update":
      case "updated":
        return "update";
      case "delete":
      case "deleted":
      case "soft_delete":
        return "delete";
      case "restore":
      case "restored":
        return "restore";
      case "upload":
      case "uploaded":
        return "upload";
      case "login":
        return "login";
      case "logout":
        return "logout";
      case "role_change":
      case "suspend":
      case "unsuspend":
        return "security";
      default:
        return "other";
    }
  }

  type BadgeVariant = "default" | "success" | "warning" | "danger" | "info" | "muted";

  function getActionVariant(action: string): BadgeVariant {
    switch (action) {
      case "create":
      case "created":
      case "restore":
      case "restored":
        return "success";
      case "delete":
      case "deleted":
      case "soft_delete":
      case "suspend":
        return "danger";
      case "update":
      case "updated":
      case "upload":
      case "uploaded":
        return "info";
      case "login":
      case "logout":
        return "muted";
      default:
        return "default";
    }
  }

  function formatAction(action: string): string {
    return action.replace(/_/g, " ");
  }

  function formatResourceType(resourceType: string): string {
    return resourceType.replace(/_/g, " ");
  }
</script>

{#if loading}
  <div class="activity-feed activity-feed--loading">
    <p class="activity-feed__loading">Loading activity...</p>
  </div>
{:else if error}
  <div class="activity-feed activity-feed--error">
    <p class="activity-feed__error">{error}</p>
  </div>
{:else if activities.length === 0}
  <div class="activity-feed activity-feed--empty">
    <p class="activity-feed__empty">{emptyMessage}</p>
  </div>
{:else}
  <ul class="activity-feed">
    {#each activities as activity}
      {@const actionType = getActionType(activity.action)}
      <li class="activity-item">
        <div class="activity-item__icon">
          {#if actionType === "create"}
            <Plus size={14} />
          {:else if actionType === "update"}
            <Pencil size={14} />
          {:else if actionType === "delete"}
            <Trash2 size={14} />
          {:else if actionType === "restore"}
            <RefreshCw size={14} />
          {:else if actionType === "upload"}
            <Upload size={14} />
          {:else if actionType === "login"}
            <LogIn size={14} />
          {:else if actionType === "logout"}
            <LogOut size={14} />
          {:else if actionType === "security"}
            <Shield size={14} />
          {:else}
            <User size={14} />
          {/if}
        </div>
        <div class="activity-item__content">
          <p class="activity-item__description">
            {#if activity.actor}
              <strong>{activity.actor.displayName ?? activity.actor.email}</strong>
            {:else}
              <strong>System</strong>
            {/if}
            <Badge variant={getActionVariant(activity.action)} size="sm">{formatAction(activity.action)}</Badge>
            <span class="activity-item__resource">{formatResourceType(activity.resourceType)}</span>
          </p>
          <p class="activity-item__time">
            {formatRelative(activity.occurredAt)}
          </p>
        </div>
      </li>
    {/each}
  </ul>
{/if}

<style>
  .activity-feed {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .activity-feed--loading,
  .activity-feed--error,
  .activity-feed--empty {
    padding: 1rem 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .activity-feed__loading,
  .activity-feed__empty {
    margin: 0;
    color: var(--text-secondary, #6b7280);
    font-size: 0.875rem;
  }

  .activity-feed__error {
    margin: 0;
    color: var(--danger, #ef4444);
    font-size: 0.875rem;
  }

  .activity-item {
    display: flex;
    gap: 0.75rem;
    align-items: flex-start;
  }

  .activity-item__icon {
    width: 1.75rem;
    height: 1.75rem;
    border-radius: 50%;
    background: var(--bg-muted, #f3f4f6);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: var(--text-secondary, #6b7280);
  }

  .activity-item__content {
    flex: 1;
    min-width: 0;
  }

  .activity-item__description {
    margin: 0;
    font-size: 0.875rem;
    color: var(--text-primary, #111827);
    display: flex;
    align-items: center;
    gap: 0.375rem;
    flex-wrap: wrap;
  }

  .activity-item__description strong {
    font-weight: 500;
  }

  .activity-item__resource {
    color: var(--text-secondary, #6b7280);
  }

  .activity-item__time {
    margin: 0.125rem 0 0;
    font-size: 0.75rem;
    color: var(--text-secondary, #6b7280);
  }
</style>
