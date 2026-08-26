import {
  PROJECT_DESCRIPTION_LABELS,
  PROJECT_DESCRIPTION_SCHEMA,
  registerNotesEditors,
} from "../notes/registrations";

registerNotesEditors(PROJECT_DESCRIPTION_SCHEMA, PROJECT_DESCRIPTION_LABELS);
