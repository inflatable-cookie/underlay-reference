import {
  applyReorderConflict,
  extractReorderConflict,
  type ReorderController,
  type ReorderableItem
} from "@decodelabs/underlay/patterns";

export interface ReorderConflictRecoveryResult {
  handled: boolean;
  message: string;
}

interface RecoverReorderConflictOptions<T extends ReorderableItem> {
  controller: ReorderController<T>;
  error: unknown;
  latestItems: readonly T[];
  entityLabel: string;
}

/**
 * Apply server reorder-conflict context to pending controller state.
 * Keeps users in reorder mode so they can review and submit again.
 */
export function recoverReorderConflict<T extends ReorderableItem>({
  controller,
  error,
  latestItems,
  entityLabel
}: RecoverReorderConflictOptions<T>): ReorderConflictRecoveryResult {
  const conflict = extractReorderConflict(error);
  if (!conflict) {
    return { handled: false, message: "" };
  }

  const resolution = applyReorderConflict(controller, conflict, latestItems);
  const changes: string[] = [];

  if (resolution.addedCount > 0) {
    changes.push(`${resolution.addedCount} ${entityLabel} added`);
  }
  if (resolution.removedCount > 0) {
    changes.push(`${resolution.removedCount} ${entityLabel} removed`);
  }
  if (resolution.unresolvedAddedIds.length > 0) {
    changes.push(`${resolution.unresolvedAddedIds.length} new item(s) need refresh`);
  }

  const suffix =
    changes.length > 0
      ? ` Applied updates: ${changes.join(", ")}.`
      : " Applied latest server state.";

  return {
    handled: true,
    message: `${conflict.message}${suffix} Review the order and save again.`
  };
}
