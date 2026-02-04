<!--
  Re-export of Underlay BatchActionBar.
  This file maintains backwards compatibility with existing imports.
-->
<script lang="ts">
  import { BatchActionBar } from "@decodelabs/underlay/components";

  interface Props {
    /** Number of selected items */
    selectedCount: number;
    /** Total number of items available for selection */
    totalCount?: number;
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
    /** Callback when select all is clicked */
    onSelectAll?: () => void;
    /** Callback when batch delete is confirmed */
    onBatchDelete?: () => void;
    /** Callback when batch status update is confirmed */
    onBatchStatusUpdate?: (status: string) => void;
  }

  let {
    selectedCount,
    totalCount = 0,
    loading = false,
    showDelete = true,
    showStatusUpdate = false,
    statusOptions = [],
    onClearSelection,
    onSelectAll,
    onBatchDelete,
    onBatchStatusUpdate
  }: Props = $props();
</script>

<BatchActionBar
  {selectedCount}
  {totalCount}
  {loading}
  {showDelete}
  {showStatusUpdate}
  {statusOptions}
  {onClearSelection}
  {onSelectAll}
  {onBatchDelete}
  {onBatchStatusUpdate}
/>
