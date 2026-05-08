import {
  registerNotesEditors,
  TASK_NOTES_LABELS,
  TASK_NOTES_SCHEMA,
} from "./registrations";

registerNotesEditors(TASK_NOTES_SCHEMA, TASK_NOTES_LABELS);
