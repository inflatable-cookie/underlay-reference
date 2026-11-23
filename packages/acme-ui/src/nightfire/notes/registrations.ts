import {
  registerBlockEditor,
  registerSchema,
  type BlockEditorComponent,
  type SchemaDefinition,
} from "@inflatable-cookie/nightfire/editor-registry";
import {
  registerBlockRenderer,
  type BlockRendererComponent,
} from "@inflatable-cookie/nightfire/render-registry";
import {
  registerBlockValidator,
  type BlockValidator,
} from "@inflatable-cookie/nightfire/validation";

import TaskChecklistEditor from "./TaskChecklistEditor.svelte";
import TaskChecklistRenderer from "./TaskChecklistRenderer.svelte";
import TaskGalleryEditor from "./TaskGalleryEditor.svelte";
import TaskGalleryRenderer from "./TaskGalleryRenderer.svelte";
import TaskNotesEditor from "./TaskNotesEditor.svelte";
import TaskNotesRenderer from "./TaskNotesRenderer.svelte";

// One notes block's editor, renderer and validator. Nightfire 0.6 removed its
// `registerNightfireBlocks` helper and the registration type with it, so this
// module wires the registries directly.
interface NotesBlockRegistration {
  schema: SchemaDefinition;
  type: string;
  label: string;
  editor: BlockEditorComponent;
  renderer: BlockRendererComponent;
  validator: BlockValidator;
}

type MarkdownBlock = {
  data?: {
    content?: string | null;
  };
};

type ChecklistItem = {
  text?: string | null;
  checked?: boolean;
};

type ChecklistBlock = {
  data?: {
    items?: ChecklistItem[];
  };
};

type GalleryPage = {
  title?: string | null;
  imageId?: string | null;
  caption?: string | null;
};

type GalleryBlock = {
  data?: {
    pages?: GalleryPage[];
  };
};

export interface NotesBlockLabels {
  markdown: string;
  checklist: string;
  gallery: string;
}

export const TASK_NOTES_SCHEMA: SchemaDefinition = {
  schema: "acme:task/notes@1",
  mode: "single",
  defaultType: "notes.markdown",
};

export const PROJECT_DESCRIPTION_SCHEMA: SchemaDefinition = {
  schema: "acme:project/description@1",
  mode: "multi",
  defaultType: "notes.markdown",
};

export const TASK_NOTES_LABELS: NotesBlockLabels = {
  markdown: "Rich Text Notes",
  checklist: "Checklist",
  gallery: "Image Gallery",
};

export const PROJECT_DESCRIPTION_LABELS: NotesBlockLabels = {
  markdown: "Rich Text",
  checklist: "Checklist",
  gallery: "Image Gallery",
};

function validateMarkdownBlock(block: unknown): unknown {
  const typed = block as MarkdownBlock;
  const content = typed?.data?.content;
  if (content !== undefined && content !== null && typeof content !== "string") {
    return null;
  }
  return block;
}

function validateChecklistBlock(block: unknown): unknown {
  const typed = block as ChecklistBlock;
  const items = typed?.data?.items;

  if (items === undefined || items === null) {
    return block;
  }

  if (!Array.isArray(items)) {
    return null;
  }

  for (let i = 0; i < items.length; i++) {
    const item = items[i];
    if (typeof item !== "object" || item === null) {
      return null;
    }
    if (item.text !== undefined && item.text !== null && typeof item.text !== "string") {
      return null;
    }
  }

  return block;
}

function validateGalleryBlock(block: unknown): unknown {
  const typed = block as GalleryBlock;
  const pages = typed?.data?.pages;

  if (pages === undefined || pages === null) {
    return block;
  }

  if (!Array.isArray(pages)) {
    return null;
  }

  for (let i = 0; i < pages.length; i++) {
    const page = pages[i];
    if (typeof page !== "object" || page === null) {
      return null;
    }
    if (page.title !== undefined && page.title !== null && typeof page.title !== "string") {
      return null;
    }
    if (page.imageId !== undefined && page.imageId !== null && typeof page.imageId !== "string") {
      return null;
    }
    if (page.caption !== undefined && page.caption !== null && typeof page.caption !== "string") {
      return null;
    }
  }

  return block;
}

export function createNotesBlockRegistrations(
  schema: SchemaDefinition,
  labels: NotesBlockLabels
): NotesBlockRegistration[] {
  return [
    {
      schema,
      type: "notes.markdown",
      label: labels.markdown,
      editor: TaskNotesEditor,
      renderer: TaskNotesRenderer,
      validator: validateMarkdownBlock,
    },
    {
      schema,
      type: "notes.checklist",
      label: labels.checklist,
      editor: TaskChecklistEditor,
      renderer: TaskChecklistRenderer,
      validator: validateChecklistBlock,
    },
    {
      schema,
      type: "notes.gallery",
      label: labels.gallery,
      editor: TaskGalleryEditor,
      renderer: TaskGalleryRenderer,
      validator: validateGalleryBlock,
    },
  ];
}

export function registerNotesEditors(
  schema: SchemaDefinition,
  labels: NotesBlockLabels
): void {
  registerSchema(schema);
  for (const registration of createNotesBlockRegistrations(schema, labels)) {
    registerBlockEditor(
      schema.schema,
      registration.type,
      registration.label,
      registration.editor
    );
  }
}

export function registerNightfireRenderers(
  schema: string,
  registrations: NotesBlockRegistration[]
): void {
  for (const registration of registrations) {
    if (registration.renderer) {
      registerBlockRenderer(schema, registration.type, registration.renderer);
    }
  }
}

export function registerNightfireValidators(
  schema: string,
  registrations: NotesBlockRegistration[]
): void {
  for (const registration of registrations) {
    if (registration.validator) {
      registerBlockValidator(schema, registration.type, registration.validator);
    }
  }
}

export function registerNotesRenderers(
  schema: SchemaDefinition,
  labels: NotesBlockLabels
): void {
  registerNightfireRenderers(schema.schema, createNotesBlockRegistrations(schema, labels));
}

export function registerNotesValidators(
  schema: SchemaDefinition,
  labels: NotesBlockLabels
): void {
  registerNightfireValidators(schema.schema, createNotesBlockRegistrations(schema, labels));
}
