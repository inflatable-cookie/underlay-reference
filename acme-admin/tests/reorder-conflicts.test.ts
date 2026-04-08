import type { ReorderController } from "@decodelabs/underlay/runtime/data";
import { describe, expect, it } from "vitest";
import { recoverReorderConflict } from "$lib/lists/reorder-conflicts";

interface Item {
  id: string;
  label: string;
}

function createController(initial: Item[]) {
  let pending = [...initial];
  return {
    get pending() {
      return pending;
    },
    set pending(value: Item[]) {
      pending = value;
    },
    mergeNewItems(items: Item[]) {
      const existing = new Set(pending.map((item) => item.id));
      pending = [...pending, ...items.filter((item) => !existing.has(item.id))];
    },
    removeItems(idsToRemove: string[]) {
      const removeSet = new Set(idsToRemove);
      pending = pending.filter((item) => !removeSet.has(item.id));
    },
  };
}

describe("recoverReorderConflict", () => {
  it("applies added and removed IDs and returns guidance message", () => {
    const controller = createController([
      { id: "a", label: "A" },
      { id: "b", label: "B" },
    ]);

    const result = recoverReorderConflict({
      controller: controller as unknown as ReorderController<Item>,
      error: {
        status: 409,
        message: "Items have changed since you started reordering.",
        raw: {
          error: {
            context: {
              added_ids: ["c"],
              removed_ids: ["b"],
            },
          },
        },
      },
      latestItems: [
        { id: "a", label: "A" },
        { id: "c", label: "C" },
      ],
      entityLabel: "project",
    });

    expect(result.handled).toBe(true);
    expect(result.message).toContain("Review the order and save again.");
    expect(controller.pending.map((item) => item.id)).toEqual(["a", "c"]);
  });

  it("does not handle non-conflict errors", () => {
    const controller = createController([{ id: "a", label: "A" }]);

    const result = recoverReorderConflict({
      controller: controller as unknown as ReorderController<Item>,
      error: { status: 400, message: "Bad request" },
      latestItems: [{ id: "a", label: "A" }],
      entityLabel: "category",
    });

    expect(result).toEqual({ handled: false, message: "" });
    expect(controller.pending.map((item) => item.id)).toEqual(["a"]);
  });
});
