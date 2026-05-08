<script lang="ts">
  import { TextInput } from "@poodle/svelte";

  type GalleryPage = {
    title?: string | null;
    imageId?: string | null;
    caption?: string | null;
  };

  type TaskGalleryBlock = {
    type: string;
    version?: string;
    hash?: string;
    data?: {
      pages?: GalleryPage[];
    };
  };

  interface Props {
    block: TaskGalleryBlock;
    onChange: (block: TaskGalleryBlock) => void;
  }

  let { block, onChange }: Props = $props();

  function getPagesFromBlock(value: TaskGalleryBlock | undefined): GalleryPage[] {
    const pages = value?.data?.pages;
    return Array.isArray(pages)
      ? pages.map((page) => ({
          title: page?.title ?? "",
          imageId: page?.imageId ?? "",
          caption: page?.caption ?? "",
        }))
      : [];
  }

  let pages = $state<GalleryPage[]>([]);

  $effect(() => {
    pages = getPagesFromBlock(block);
  });

  function emit(nextPages: GalleryPage[]) {
    pages = nextPages;
    onChange({
      type: block?.type ?? "notes.gallery",
      version: block?.version ?? "initial",
      hash: block?.hash ?? "",
      data: { pages: nextPages },
    });
  }

  function addPage() {
    emit([...pages, { title: "", imageId: "", caption: "" }]);
  }

  function removePage(index: number) {
    const next = [...pages];
    next.splice(index, 1);
    emit(next);
  }

  function updatePage(index: number, patch: Partial<GalleryPage>) {
    const next = [...pages];
    next[index] = { ...next[index], ...patch };
    emit(next);
  }
</script>

<div class="task-gallery-editor">
  <p class="hint">
    Add one or more media-backed gallery cards. Each page stores a nested
    <code>imageId</code>, which is used by the reference media-usage sync path.
  </p>

  <div class="pages">
    {#if pages.length === 0}
      <p class="empty">No gallery pages yet.</p>
    {/if}

    {#each pages as page, index}
      <div class="page-card">
        <div class="page-grid">
          <TextInput
            id={`gallery-page-title-${index}`}
            type="text"
            placeholder="Page title"
            value={page.title ?? ""}
            on:valueChange={(event: CustomEvent<{ value: string }>) =>
              updatePage(index, { title: event.detail.value })}
          />
          <TextInput
            id={`gallery-page-image-${index}`}
            type="text"
            placeholder="Media ID (UUID)"
            value={page.imageId ?? ""}
            on:valueChange={(event: CustomEvent<{ value: string }>) =>
              updatePage(index, { imageId: event.detail.value })}
          />
          <TextInput
            id={`gallery-page-caption-${index}`}
            type="text"
            placeholder="Caption"
            value={page.caption ?? ""}
            on:valueChange={(event: CustomEvent<{ value: string }>) =>
              updatePage(index, { caption: event.detail.value })}
          />
        </div>

        <button type="button" class="remove" onclick={() => removePage(index)}>
          Remove page
        </button>
      </div>
    {/each}
  </div>

  <button type="button" class="add" onclick={addPage}>
    + Add gallery page
  </button>
</div>

<style>
  .task-gallery-editor {
    display: flex;
    flex-direction: column;
    gap: var(--underlay-space-3, 0.75rem);
  }

  .hint {
    margin: 0;
    font-size: calc(1em * var(--underlay-font-scale-xs, 0.85));
    color: var(--underlay-color-text-muted, rgba(148, 163, 184, 0.7));
  }

  .pages {
    display: flex;
    flex-direction: column;
    gap: var(--underlay-space-3, 0.75rem);
  }

  .page-card {
    display: flex;
    flex-direction: column;
    gap: var(--underlay-space-2, 0.5rem);
    padding: var(--underlay-space-3, 0.75rem);
    border: 1px solid var(--underlay-color-border-subtle, rgba(148, 163, 184, 0.2));
    border-radius: var(--underlay-radius-md, 0.5rem);
    background: var(--underlay-color-surface-muted, rgba(255, 255, 255, 0.02));
  }

  .page-grid {
    display: grid;
    gap: var(--underlay-space-2, 0.5rem);
  }

  .empty {
    margin: 0;
    padding: var(--underlay-space-3, 0.75rem);
    color: var(--underlay-color-text-muted, rgba(148, 163, 184, 0.7));
    font-style: italic;
  }

  button {
    align-self: flex-start;
    padding: var(--underlay-space-2, 0.5rem) var(--underlay-space-3, 0.75rem);
    border: 1px solid var(--underlay-color-border, rgba(148, 163, 184, 0.35));
    border-radius: var(--underlay-radius-pill, 9999px);
    background: transparent;
    color: var(--underlay-color-text, #e5e7eb);
    cursor: pointer;
  }

  button:hover {
    background: var(--underlay-color-field-bg, rgba(148, 163, 184, 0.08));
  }

  .remove {
    color: var(--underlay-color-danger, #ef4444);
    border-color: var(--underlay-color-danger, #ef4444);
  }
</style>
