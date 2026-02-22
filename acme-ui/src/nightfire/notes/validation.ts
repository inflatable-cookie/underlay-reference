import { registerBlockValidator } from "@decodelabs/underlay/nightfire";

const TASK_NOTES_SCHEMA = "acme:task/notes@1";

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

// Markdown notes validation - content is optional but if present must be string
registerBlockValidator(TASK_NOTES_SCHEMA, "notes.markdown", (block: unknown) => {
  const typed = block as MarkdownBlock;
  const content = typed?.data?.content;
  if (content !== undefined && content !== null && typeof content !== "string") {
    return { valid: false, errors: ["Content must be a string"] };
  }
  return { valid: true, errors: [] };
});

// Checklist validation - items must be array of valid checklist items
registerBlockValidator(TASK_NOTES_SCHEMA, "notes.checklist", (block: unknown) => {
  const typed = block as ChecklistBlock;
  const items = typed?.data?.items;

  if (items === undefined || items === null) {
    return { valid: true, errors: [] };
  }

  if (!Array.isArray(items)) {
    return { valid: false, errors: ["Items must be an array"] };
  }

  for (let i = 0; i < items.length; i++) {
    const item = items[i];
    if (typeof item !== "object" || item === null) {
      return { valid: false, errors: [`Item ${i + 1} is invalid`] };
    }
    if (item.text !== undefined && item.text !== null && typeof item.text !== "string") {
      return { valid: false, errors: [`Item ${i + 1} text must be a string`] };
    }
  }

  return { valid: true, errors: [] };
});
