import {
  PROJECT_DESCRIPTION_LABELS,
  PROJECT_DESCRIPTION_SCHEMA,
  registerNotesRenderers,
} from "../notes/registrations";

registerNotesRenderers(PROJECT_DESCRIPTION_SCHEMA, PROJECT_DESCRIPTION_LABELS);
