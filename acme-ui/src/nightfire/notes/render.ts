import {
  registerNotesRenderers,
  TASK_NOTES_LABELS,
  TASK_NOTES_SCHEMA,
} from "./registrations";

registerNotesRenderers(TASK_NOTES_SCHEMA, TASK_NOTES_LABELS);
