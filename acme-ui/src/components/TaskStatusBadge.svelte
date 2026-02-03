<script lang="ts">
  import type { Snippet } from "svelte";

  type TaskStatus = "pending" | "in_progress" | "completed" | "cancelled";

  interface Props {
    status: TaskStatus;
    size?: "sm" | "md" | "lg";
    children?: Snippet;
  }

  let { status, size = "md", children }: Props = $props();

  const labels: Record<TaskStatus, string> = {
    pending: "Pending",
    in_progress: "In Progress",
    completed: "Completed",
    cancelled: "Cancelled"
  };

  const label = $derived(labels[status] ?? status);
</script>

<span class="task-status-badge {status} size-{size}">
  <span class="indicator"></span>
  {#if children}
    {@render children()}
  {:else}
    {label}
  {/if}
</span>

<style>
  .task-status-badge {
    display: inline-flex;
    align-items: center;
    gap: var(--underlay-space-1, 0.25rem);
    padding: var(--underlay-space-1, 0.25rem) var(--underlay-space-2, 0.5rem);
    border-radius: var(--underlay-radius-pill, 9999px);
    font-weight: 500;
    white-space: nowrap;
  }

  /* Sizes */
  .size-sm {
    font-size: calc(1em * var(--underlay-font-scale-xxs, 0.75));
    padding: 0.125rem var(--underlay-space-1, 0.25rem);
  }

  .size-md {
    font-size: calc(1em * var(--underlay-font-scale-xs, 0.85));
  }

  .size-lg {
    font-size: calc(1em * var(--underlay-font-scale-sm, 0.9));
    padding: var(--underlay-space-1, 0.25rem) var(--underlay-space-3, 0.75rem);
  }

  .indicator {
    width: 0.5em;
    height: 0.5em;
    border-radius: 50%;
    flex-shrink: 0;
  }

  /* Status variants */
  .pending {
    background: rgba(148, 163, 184, 0.15);
    color: var(--underlay-color-text-muted, rgba(148, 163, 184, 0.9));
  }

  .pending .indicator {
    background: var(--underlay-color-text-muted, rgba(148, 163, 184, 0.7));
  }

  .in_progress {
    background: rgba(59, 130, 246, 0.15);
    color: var(--underlay-color-primary, #3b82f6);
  }

  .in_progress .indicator {
    background: var(--underlay-color-primary, #3b82f6);
    animation: pulse 2s ease-in-out infinite;
  }

  .completed {
    background: rgba(34, 197, 94, 0.15);
    color: var(--underlay-color-success, #22c55e);
  }

  .completed .indicator {
    background: var(--underlay-color-success, #22c55e);
  }

  .cancelled {
    background: rgba(239, 68, 68, 0.15);
    color: var(--underlay-color-danger, #ef4444);
  }

  .cancelled .indicator {
    background: var(--underlay-color-danger, #ef4444);
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }
</style>
