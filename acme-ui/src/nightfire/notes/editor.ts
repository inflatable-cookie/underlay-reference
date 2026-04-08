import TaskNotesEditor from "./TaskNotesEditor.svelte";
import TaskChecklistEditor from "./TaskChecklistEditor.svelte";

import {
  registerSchema,
  registerBlockEditor,
  type SchemaDefinition
} from "@decodelabs/underlay/nightfire/editor-registry";

// Task notes schema - supports multiple block types for rich task content.
const TASK_NOTES_SCHEMA: SchemaDefinition = {
  schema: "acme:task/notes@1",
  mode: "single",
  defaultType: "notes.markdown"
};

registerSchema(TASK_NOTES_SCHEMA);

// Markdown notes block - simple rich text content
registerBlockEditor(
  TASK_NOTES_SCHEMA.schema,
  "notes.markdown",
  "Rich Text Notes",
  TaskNotesEditor
);

// Checklist block - interactive checklist items
registerBlockEditor(
  TASK_NOTES_SCHEMA.schema,
  "notes.checklist",
  "Checklist",
  TaskChecklistEditor
);
