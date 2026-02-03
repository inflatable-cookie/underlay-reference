<script lang="ts">
  type ChecklistItem = {
    text?: string | null;
    checked?: boolean;
  };

  type TaskChecklistBlock = {
    data?: {
      items?: ChecklistItem[];
    };
  };

  interface Props {
    block: TaskChecklistBlock;
  }

  let { block }: Props = $props();

  const items: ChecklistItem[] = $derived(
    Array.isArray(block?.data?.items) ? block.data!.items : []
  );

  const completedCount = $derived(items.filter(i => i.checked).length);
  const totalCount = $derived(items.length);
  const progressPercent = $derived(totalCount > 0 ? Math.round((completedCount / totalCount) * 100) : 0);
</script>

{#if items.length === 0}
  <div class="task-checklist empty">
    <p>No checklist items.</p>
  </div>
{:else}
  <div class="task-checklist">
    <div class="progress-bar">
      <div class="progress-fill" style="width: {progressPercent}%"></div>
    </div>
    <p class="progress-text">{completedCount} of {totalCount} complete</p>

    <ul class="checklist-items">
      {#each items as item}
        <li class="checklist-item" class:checked={item.checked}>
          <span class="check-icon">
            {#if item.checked}
              ✓
            {:else}
              ○
            {/if}
          </span>
          <span class="item-text">{item.text || "(empty)"}</span>
        </li>
      {/each}
    </ul>
  </div>
{/if}

<style>
  .task-checklist {
    display: flex;
    flex-direction: column;
    gap: var(--underlay-space-3, 0.75rem);
  }

  .progress-bar {
    height: 6px;
    background: var(--underlay-color-surface-muted, rgba(255, 255, 255, 0.1));
    border-radius: var(--underlay-radius-pill, 9999px);
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--underlay-color-success, #22c55e);
    border-radius: var(--underlay-radius-pill, 9999px);
    transition: width 0.3s ease;
  }

  .progress-text {
    margin: 0;
    font-size: calc(1em * var(--underlay-font-scale-xs, 0.85));
    color: var(--underlay-color-text-muted, rgba(148, 163, 184, 0.7));
  }

  .checklist-items {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--underlay-space-2, 0.5rem);
  }

  .checklist-item {
    display: flex;
    align-items: flex-start;
    gap: var(--underlay-space-2, 0.5rem);
    padding: var(--underlay-space-2, 0.5rem);
    background: var(--underlay-color-surface-muted, rgba(255, 255, 255, 0.02));
    border-radius: var(--underlay-radius-md, 0.5rem);
    color: var(--underlay-color-text, #e5e7eb);
  }

  .checklist-item.checked {
    color: var(--underlay-color-text-muted, rgba(148, 163, 184, 0.7));
  }

  .checklist-item.checked .item-text {
    text-decoration: line-through;
  }

  .check-icon {
    flex-shrink: 0;
    width: 1.25rem;
    height: 1.25rem;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.875rem;
    color: var(--underlay-color-text-muted, rgba(148, 163, 184, 0.5));
  }

  .checklist-item.checked .check-icon {
    color: var(--underlay-color-success, #22c55e);
  }

  .item-text {
    flex: 1;
    line-height: 1.4;
  }

  .empty {
    padding: var(--underlay-space-3, 0.75rem);
    color: var(--underlay-color-text-muted, rgba(148, 163, 184, 0.7));
    font-style: italic;
  }

  .empty p {
    margin: 0;
  }
</style>
