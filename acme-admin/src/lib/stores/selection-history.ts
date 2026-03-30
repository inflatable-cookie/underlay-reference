import { createSelectionHistory } from "@decodelabs/underlay/runtime";
/**
 * Selection history stores for RelationSelector components.
 *
 * These track recently selected items to provide better suggestions
 * when users open relation pickers.
 */

/**
 * History of recently selected categories.
 * Used in project forms when selecting a category.
 */
export const categorySelectionHistory = createSelectionHistory(
  "acme.categories",
  {
    maxItems: 10,
    storageType: "local",
  },
);

/**
 * History of recently selected projects.
 * Used when selecting projects for tasks or other relations.
 */
export const projectSelectionHistory = createSelectionHistory("acme.projects", {
  maxItems: 10,
  storageType: "local",
});

/**
 * History of recently selected labels.
 * Used in task forms when assigning labels.
 */
export const labelSelectionHistory = createSelectionHistory("acme.labels", {
  maxItems: 20,
  storageType: "local",
});
