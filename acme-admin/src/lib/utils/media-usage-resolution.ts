import {
  parseNightfireMediaLocator,
  resolveNightfireMediaLocator,
} from "@inflatable-cookie/underlay/nightfire/media-locator";
import type { NightfireValue } from "@inflatable-cookie/underlay/nightfire/validation";
import type { MediaUsage, Task } from "@api-client";

function decodePointerSegment(segment: string): string {
  return segment.replace(/~1/g, "/").replace(/~0/g, "~");
}

function resolveJsonPointer(value: unknown, pointer: string): unknown {
  if (pointer === "") {
    return value;
  }

  return pointer
    .slice(1)
    .split("/")
    .map(decodePointerSegment)
    .reduce<unknown>((current, segment) => {
      if (current === null || typeof current !== "object") {
        return undefined;
      }

      if (Array.isArray(current)) {
        const index = Number(segment);
        return Number.isInteger(index) ? current[index] : undefined;
      }

      return (current as Record<string, unknown>)[segment];
    }, value);
}

export function resolveMediaUsageInTaskNotes(
  usage: MediaUsage,
  task: Task | null | undefined,
): unknown {
  if (
    usage.usedByType !== "task" ||
    usage.ownerField !== "notes" ||
    !task?.notes
  ) {
    return undefined;
  }

  const notes = task.notes as NightfireValue;

  if (usage.locatorKind === "block_id") {
    try {
      const locator = parseNightfireMediaLocator(usage.locatorKey);
      return resolveNightfireMediaLocator(notes, locator);
    } catch {
      return undefined;
    }
  }

  if (usage.locatorKind === "path") {
    return resolveJsonPointer(notes, usage.locatorKey);
  }

  if (usage.locatorKind === "field") {
    return usage.ownerField
      ? (notes as unknown as Record<string, unknown>)[usage.ownerField]
      : undefined;
  }

  return undefined;
}

export function formatResolvedUsageValue(value: unknown): string | null {
  if (typeof value === "string") {
    return value;
  }

  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }

  if (value === null) {
    return "null";
  }

  if (value === undefined) {
    return null;
  }

  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}
