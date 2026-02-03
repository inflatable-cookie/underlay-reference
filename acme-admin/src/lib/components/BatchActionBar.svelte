<script lang="ts">
  /**
   * BatchActionBar - A fixed toolbar that appears when items are selected.
   *
   * Shows selection count and action buttons for batch operations.
   */
  import { Button, AlertDialog, Dialog } from "@decodelabs/underlay/components";
  import X from "lucide-svelte/icons/x";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import CheckCheck from "lucide-svelte/icons/check-check";

  interface Props {
    /** Number of selected items */
    selectedCount: number;
    /** Whether a batch operation is in progress */
    loading?: boolean;
    /** Show batch delete action */
    showDelete?: boolean;
    /** Show batch status update action (for tasks) */
    showStatusUpdate?: boolean;
    /** Available status options for status update */
    statusOptions?: { value: string; label: string }[];
    /** Callback when clear selection is clicked */
    onClearSelection: () => void;
    /** Callback when batch delete is confirmed */
    onBatchDelete?: () => void;
    /** Callback when batch status update is confirmed */
    onBatchStatusUpdate?: (status: string) => void;
  }

  let {
    selectedCount,
    loading = false,
    showDelete = true,
    showStatusUpdate = false,
    statusOptions = [],
    onClearSelection,
    onBatchDelete,
    onBatchStatusUpdate
  }: Props = $props();

  let showDeleteConfirm = $state(false);
  let showStatusModal = $state(false);
  let selectedStatus = $state("");

  function handleDeleteClick() {
    showDeleteConfirm = true;
  }

  function confirmDelete() {
    showDeleteConfirm = false;
    onBatchDelete?.();
  }

  function handleStatusClick() {
    showStatusModal = true;
  }

  function confirmStatusUpdate() {
    if (selectedStatus) {
      showStatusModal = false;
      onBatchStatusUpdate?.(selectedStatus);
      selectedStatus = "";
    }
  }
</script>

{#if selectedCount > 0}
  <div class="batch-action-bar">
    <div class="selection-info">
      <span class="count">{selectedCount}</span>
      <span class="label">{selectedCount === 1 ? "item" : "items"} selected</span>
    </div>

    <div class="actions">
      {#if showStatusUpdate && statusOptions.length > 0}
        <Button
          type="button"
          variant="subtle"
          size="sm"
          onclick={handleStatusClick}
          disabled={loading}
        >
          <CheckCheck size={16} />
          Update Status
        </Button>
      {/if}

      {#if showDelete}
        <Button
          type="button"
          variant="danger"
          size="sm"
          onclick={handleDeleteClick}
          disabled={loading}
        >
          <Trash2 size={16} />
          Delete
        </Button>
      {/if}

      <Button
        type="button"
        variant="subtle"
        size="sm"
        onclick={onClearSelection}
        disabled={loading}
      >
        <X size={16} />
        Clear
      </Button>
    </div>
  </div>
{/if}

<!-- Delete confirmation -->
<AlertDialog
  bind:open={showDeleteConfirm}
  showTrigger={false}
  title="Confirm Delete"
  description={`Are you sure you want to delete ${selectedCount} ${selectedCount === 1 ? "item" : "items"}? This action cannot be undone.`}
  confirmLabel={`Delete ${selectedCount} ${selectedCount === 1 ? "item" : "items"}`}
  onConfirm={confirmDelete}
/>

<!-- Status update dialog -->
{#if showStatusUpdate}
  <Dialog
    bind:open={showStatusModal}
    title="Update Status"
  >
    <p class="confirm-text">
      Update status for <strong>{selectedCount}</strong> {selectedCount === 1 ? "item" : "items"}:
    </p>
    <div class="status-options">
      {#each statusOptions as option}
        <label class="status-option">
          <input
            type="radio"
            name="status"
            value={option.value}
            bind:group={selectedStatus}
          />
          <span>{option.label}</span>
        </label>
      {/each}
    </div>
    <div class="dialog-actions">
      <Button
        type="button"
        variant="subtle"
        onclick={() => showStatusModal = false}
      >
        Cancel
      </Button>
      <Button
        type="button"
        variant="primary"
        onclick={confirmStatusUpdate}
        disabled={!selectedStatus}
      >
        Update Status
      </Button>
    </div>
  </Dialog>
{/if}

<style>
  .batch-action-bar {
    position: fixed;
    bottom: 1.5rem;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 0.75rem 1.25rem;
    background: var(--surface-elevated, #1f2937);
    border: 1px solid var(--border-color, #374151);
    border-radius: 0.75rem;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
    z-index: 100;
  }

  .selection-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.75rem;
    height: 1.75rem;
    padding: 0 0.5rem;
    background: var(--accent-color, #3b82f6);
    color: white;
    font-weight: 600;
    font-size: 0.875rem;
    border-radius: 9999px;
  }

  .label {
    color: var(--text-secondary, #9ca3af);
    font-size: 0.875rem;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .confirm-text {
    margin: 0 0 1rem;
  }

  .status-options {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .status-option {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-radius: 0.375rem;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .status-option:hover {
    background: var(--surface-hover, #374151);
  }

  .status-option input {
    accent-color: var(--accent-color, #3b82f6);
  }

  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1.5rem;
  }
</style>
