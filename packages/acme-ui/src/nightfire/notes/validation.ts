import {
  registerNotesValidators,
  TASK_NOTES_LABELS,
  TASK_NOTES_SCHEMA,
} from "./registrations";

registerNotesValidators(TASK_NOTES_SCHEMA, TASK_NOTES_LABELS);
