import {
  PROJECT_DESCRIPTION_LABELS,
  PROJECT_DESCRIPTION_SCHEMA,
  registerNotesValidators,
} from "../notes/registrations";

registerNotesValidators(PROJECT_DESCRIPTION_SCHEMA, PROJECT_DESCRIPTION_LABELS);
