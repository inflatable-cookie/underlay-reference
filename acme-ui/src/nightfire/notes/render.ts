import TaskNotesRenderer from "./TaskNotesRenderer.svelte";
import TaskChecklistRenderer from "./TaskChecklistRenderer.svelte";

import { registerBlockRenderer } from "@decodelabs/underlay/nightfire/render-registry";

const TASK_NOTES_SCHEMA = "acme:task/notes@1";

// Block renderers for task notes content types.

registerBlockRenderer(
  TASK_NOTES_SCHEMA,
  "notes.markdown",
  TaskNotesRenderer as unknown as Parameters<typeof registerBlockRenderer>[2]
);
registerBlockRenderer(
  TASK_NOTES_SCHEMA,
  "notes.checklist",
  TaskChecklistRenderer as unknown as Parameters<typeof registerBlockRenderer>[2]
);
