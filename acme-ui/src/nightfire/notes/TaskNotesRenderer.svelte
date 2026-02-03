<script lang="ts">
  import { marked } from "marked";

  type TaskNotesBlock = {
    data?: {
      content?: string | null;
    };
  };

  interface Props {
    block: TaskNotesBlock;
  }

  let { block }: Props = $props();

  const content = $derived(block?.data?.content ?? "");
  const html = $derived(content ? marked.parse(content) : "");
</script>

{#if content}
  <div class="task-notes">
    {@html html}
  </div>
{:else}
  <div class="task-notes empty">
    <p>No notes added yet.</p>
  </div>
{/if}

<style>
  .task-notes {
    font-size: 0.9375rem;
    line-height: 1.6;
    color: var(--underlay-color-text, #e5e7eb);
  }

  .task-notes :global(h1),
  .task-notes :global(h2),
  .task-notes :global(h3),
  .task-notes :global(h4) {
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    font-weight: 600;
  }

  .task-notes :global(h1:first-child),
  .task-notes :global(h2:first-child),
  .task-notes :global(h3:first-child),
  .task-notes :global(h4:first-child) {
    margin-top: 0;
  }

  .task-notes :global(p) {
    margin: 0 0 1em;
  }

  .task-notes :global(ul),
  .task-notes :global(ol) {
    margin: 0 0 1em;
    padding-left: 1.5em;
  }

  .task-notes :global(code) {
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Monaco, Consolas, monospace;
    font-size: 0.875em;
    background: var(--underlay-color-surface-muted, rgba(255, 255, 255, 0.05));
    padding: 0.125em 0.375em;
    border-radius: var(--underlay-radius-sm, 0.25rem);
  }

  .task-notes :global(pre) {
    margin: 0 0 1em;
    padding: var(--underlay-space-3, 0.75rem);
    background: var(--underlay-color-surface-muted, rgba(255, 255, 255, 0.05));
    border-radius: var(--underlay-radius-md, 0.5rem);
    overflow-x: auto;
  }

  .task-notes :global(pre code) {
    background: none;
    padding: 0;
  }

  .task-notes :global(blockquote) {
    margin: 0 0 1em;
    padding-left: var(--underlay-space-3, 0.75rem);
    border-left: 3px solid var(--underlay-color-primary, #3b82f6);
    color: var(--underlay-color-text-muted, rgba(148, 163, 184, 0.85));
  }

  .empty {
    padding: var(--underlay-space-3, 0.75rem);
    color: var(--underlay-color-text-muted, rgba(148, 163, 184, 0.7));
    font-style: italic;
  }

  .empty p {
    margin: 0;
  }
</style>
