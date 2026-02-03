<script lang="ts">
  import type { Snippet } from "svelte";

  type TaskPriority = "low" | "medium" | "high" | "urgent";

  interface Props {
    priority: TaskPriority;
    size?: "sm" | "md" | "lg";
    showLabel?: boolean;
    children?: Snippet;
  }

  let { priority, size = "md", showLabel = true, children }: Props = $props();

  const labels: Record<TaskPriority, string> = {
    low: "Low",
    medium: "Medium",
    high: "High",
    urgent: "Urgent"
  };

  const icons: Record<TaskPriority, string> = {
    low: "↓",
    medium: "→",
    high: "↑",
    urgent: "⚡"
  };

  const label = $derived(labels[priority] ?? priority);
  const icon = $derived(icons[priority] ?? "•");
</script>

<span class="priority-badge {priority} size-{size}">
  <span class="icon">{icon}</span>
  {#if children}
    {@render children()}
  {:else if showLabel}
    {label}
  {/if}
</span>

<style>
  .priority-badge {
    display: inline-flex;
    align-items: center;
    gap: var(--underlay-space-1, 0.25rem);
    padding: var(--underlay-space-1, 0.25rem) var(--underlay-space-2, 0.5rem);
    border-radius: var(--underlay-radius-sm, 0.25rem);
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

  .icon {
    font-size: 1em;
  }

  /* Priority variants */
  .low {
    background: rgba(148, 163, 184, 0.1);
    color: var(--underlay-color-text-muted, rgba(148, 163, 184, 0.8));
  }

  .medium {
    background: rgba(59, 130, 246, 0.1);
    color: var(--underlay-color-primary, #3b82f6);
  }

  .high {
    background: rgba(249, 115, 22, 0.15);
    color: #f97316;
  }

  .urgent {
    background: rgba(239, 68, 68, 0.15);
    color: var(--underlay-color-danger, #ef4444);
    font-weight: 600;
  }

  .urgent .icon {
    animation: flash 1s ease-in-out infinite;
  }

  @keyframes flash {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.6;
    }
  }
</style>
