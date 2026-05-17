<script lang="ts">
  import { TextInput } from "@poodle/svelte";

  type ChecklistItem = {
    text: string;
    checked: boolean;
  };

  type TaskChecklistBlock = {
    type: string;
    version?: string;
    hash?: string;
    data?: {
      items?: ChecklistItem[];
    };
  };

  interface Props {
    block: TaskChecklistBlock;
    onChange: (block: TaskChecklistBlock) => void;
  }

  let { block, onChange }: Props = $props();

  function getItemsFromBlock(b: TaskChecklistBlock | undefined): ChecklistItem[] {
    const items = b?.data?.items;
    return Array.isArray(items)
      ? items.map(i => ({ text: i.text ?? "", checked: Boolean(i.checked) }))
      : [];
  }

  let items = $state<ChecklistItem[]>([]);

  $effect(() => {
    items = getItemsFromBlock(block);
  });

  function emit(nextItems: ChecklistItem[]) {
    items = nextItems;
    onChange({
      type: block?.type ?? "notes.checklist",
      version: block?.version ?? "initial",
      hash: block?.hash ?? "",
      data: { items: nextItems }
    });
  }

  function addItem() {
    emit([...items, { text: "", checked: false }]);
  }

  function removeItem(index: number) {
    const next = [...items];
    next.splice(index, 1);
    emit(next);
  }

  function moveItem(from: number, to: number) {
    if (from === to || from < 0 || to < 0 || from >= items.length || to >= items.length) {
      return;
    }
    const next = [...items];
    const [moved] = next.splice(from, 1);
    next.splice(to, 0, moved);
    emit(next);
  }

  function updateItemText(index: number, text: string) {
    const next = [...items];
    next[index] = { ...next[index], text };
    emit(next);
  }

  function toggleItemChecked(index: number) {
    const next = [...items];
    next[index] = { ...next[index], checked: !next[index].checked };
    emit(next);
  }
</script>

<div class="task-checklist-editor">
  <p class="hint">
    Create a checklist of items. Check off items as they are completed.
  </p>

  <div class="items">
    {#if items.length === 0}
      <p class="empty">No checklist items yet.</p>
    {/if}

    {#each items as item, index}
      <div class="item-row">
        <input
          type="checkbox"
          checked={item.checked}
          onchange={() => toggleItemChecked(index)}
          class="checkbox"
        />
        <TextInput
          id={`checklist-item-${index}`}
          type="text"
          placeholder="Checklist item..."
          value={item.text}
          onValueChange={(nextValue) => updateItemText(index, nextValue)}
        />
        <div class="item-controls">
          <button
            type="button"
            class="move"
            onclick={() => moveItem(index, index - 1)}
            disabled={index === 0}
            title="Move up"
          >
            ↑
          </button>
          <button
            type="button"
            class="move"
            onclick={() => moveItem(index, index + 1)}
            disabled={index === items.length - 1}
            title="Move down"
          >
            ↓
          </button>
          <button
            type="button"
            class="remove"
            onclick={() => removeItem(index)}
            title="Remove"
          >
            ×
          </button>
        </div>
      </div>
    {/each}
  </div>

  <button type="button" class="add" onclick={addItem}>
    + Add item
  </button>
</div>

<style>
  .task-checklist-editor {
    display: flex;
    flex-direction: column;
    gap: var(--underlay-space-3, 0.75rem);
  }

  .hint {
    margin: 0;
    font-size: calc(1em * var(--underlay-font-scale-xs, 0.85));
    color: var(--underlay-color-text-muted, rgba(148, 163, 184, 0.7));
  }

  .items {
    display: flex;
    flex-direction: column;
    gap: var(--underlay-space-2, 0.5rem);
  }

  .empty {
    margin: 0;
    padding: var(--underlay-space-3, 0.75rem);
    color: var(--underlay-color-text-muted, rgba(148, 163, 184, 0.7));
    font-style: italic;
  }

  .item-row {
    display: flex;
    align-items: center;
    gap: var(--underlay-space-2, 0.5rem);
    padding: var(--underlay-space-2, 0.5rem);
    background: var(--underlay-color-surface-muted, rgba(255, 255, 255, 0.02));
    border: 1px solid var(--underlay-color-border-subtle, rgba(148, 163, 184, 0.2));
    border-radius: var(--underlay-radius-md, 0.5rem);
  }

  .checkbox {
    width: 1.25rem;
    height: 1.25rem;
    cursor: pointer;
    accent-color: var(--underlay-color-primary, #3b82f6);
  }

  .item-controls {
    display: flex;
    gap: var(--underlay-space-1, 0.25rem);
    flex-shrink: 0;
  }

  button {
    padding: var(--underlay-space-1, 0.25rem) var(--underlay-space-2, 0.5rem);
    border: 1px solid var(--underlay-color-border, rgba(148, 163, 184, 0.35));
    border-radius: var(--underlay-radius-sm, 0.25rem);
    background: transparent;
    color: var(--underlay-color-text, #e5e7eb);
    font-size: calc(1em * var(--underlay-font-scale-xxs, 0.75));
    cursor: pointer;
  }

  button:hover:not(:disabled) {
    background: var(--underlay-color-field-bg, rgba(148, 163, 184, 0.08));
  }

  button:disabled {
    opacity: 0.4;
    cursor: default;
  }

  button.move {
    padding: var(--underlay-space-1, 0.25rem);
    min-width: 1.5rem;
  }

  button.remove {
    color: var(--underlay-color-danger, #ef4444);
    border-color: var(--underlay-color-danger, #ef4444);
    font-size: 1rem;
    font-weight: bold;
    line-height: 1;
  }

  button.remove:hover {
    background: rgba(239, 68, 68, 0.1);
  }

  button.add {
    align-self: flex-start;
    padding: var(--underlay-space-2, 0.5rem) var(--underlay-space-3, 0.75rem);
    border-radius: var(--underlay-radius-pill, 9999px);
    border: 1px solid var(--underlay-color-border-strong, rgba(148, 163, 184, 0.5));
  }
</style>
