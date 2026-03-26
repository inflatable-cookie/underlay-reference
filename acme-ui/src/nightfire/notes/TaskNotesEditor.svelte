<script lang="ts">
  import { MarkdownEditor } from "@poodle/svelte-composites";

  type TaskNotesBlock = {
    type: string;
    version?: string;
    hash?: string;
    data?: {
      content?: string | null;
    };
  };

  interface Props {
    block: TaskNotesBlock;
    onChange: (block: TaskNotesBlock) => void;
  }

  let { block, onChange }: Props = $props();

  let content = $state("");

  $effect(() => {
    content = block?.data?.content ?? "";
  });

  function handleChange(text: string) {
    content = text;
    onChange({
      type: block?.type ?? "notes.markdown",
      version: block?.version ?? "initial",
      hash: block?.hash ?? "",
      data: { content: text }
    });
  }
</script>

<div class="task-notes-editor">
  <p class="hint">
    Add notes to this task using markdown formatting.
  </p>

  <MarkdownEditor
    placeholder="Write your notes here..."
    value={content}
    on:change={(event) => handleChange(event.detail.value)}
  />
</div>

<style>
  .task-notes-editor {
    display: flex;
    flex-direction: column;
    gap: var(--underlay-space-3, 0.75rem);
  }

  .hint {
    margin: 0;
    font-size: calc(1em * var(--underlay-font-scale-xs, 0.85));
    color: var(--underlay-color-text-muted, rgba(148, 163, 184, 0.7));
  }
</style>
