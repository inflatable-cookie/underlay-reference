<script lang="ts">
  import { Code } from "@inflatable-cookie/poodle-svelte";

  type GalleryPage = {
    title?: string | null;
    imageId?: string | null;
    caption?: string | null;
  };

  type TaskGalleryBlock = {
    data?: {
      pages?: GalleryPage[];
    };
  };

  interface Props {
    block: TaskGalleryBlock;
  }

  let { block }: Props = $props();

  const pages: GalleryPage[] = $derived(
    Array.isArray(block?.data?.pages) ? block.data!.pages : []
  );
</script>

{#if pages.length === 0}
  <div class="task-gallery empty">
    <p>No gallery pages.</p>
  </div>
{:else}
  <div class="task-gallery">
    {#each pages as page}
      <article class="page-card">
        <h4>{page.title || "Untitled image"}</h4>
        {#if page.caption}
          <p>{page.caption}</p>
        {/if}
        {#if page.imageId}
          <Code inline source={page.imageId} />
        {:else}
          <span class="missing">No media selected</span>
        {/if}
      </article>
    {/each}
  </div>
{/if}

<style>
  .task-gallery {
    display: grid;
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

  h4,
  p {
    margin: 0;
  }

  .missing,
  .empty {
    color: var(--underlay-color-text-muted, rgba(148, 163, 184, 0.7));
    font-style: italic;
  }

  .empty p {
    margin: 0;
  }
</style>
