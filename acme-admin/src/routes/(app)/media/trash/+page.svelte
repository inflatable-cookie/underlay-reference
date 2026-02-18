<script lang="ts">
  import { goto } from "$app/navigation";
  import {
    PageHeader,
    useToasts,
    useAuthenticatedData,
    getMediaKindLabel,
    getMediaKindAccent,
    formatFileSize
  } from "@decodelabs/underlay/patterns";
  import {
    Button,
    EmptyState,
    FormError,
    ListGrid,
    ListCard,
    MediaThumbnail,
    Pill,
    PageLoading,
    ConfirmAction
  } from "@decodelabs/underlay/components";
  import { mediaCommands, type MediaSummary } from "acme-client";
  import { auth } from "$lib/stores/auth";
  import RotateCcw from "lucide-svelte/icons/rotate-ccw";
  import Trash2 from "lucide-svelte/icons/trash-2";

  const toastStore = useToasts();

  // Fetch trashed media
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const media = await mediaCommands.listMediaTrash(fetch, token);
      return { media };
    },
    {
      defaultValue: { media: [] as MediaSummary[] }
    }
  );

  async function handleRestore(mediaId: string) {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await mediaCommands.restoreMedia(mediaId, fetch, token);
      toastStore.push({ variant: "success", message: "Media restored" });
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to restore media";
      toastStore.push({ variant: "error", message });
    }
  }

  async function handlePurge(mediaId: string) {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await mediaCommands.purgeMedia(mediaId, fetch, token);
      toastStore.push({ variant: "success", message: "Media permanently deleted" });
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete media";
      toastStore.push({ variant: "error", message });
    }
  }

</script>

<PageHeader section="Media Trash" backHref="/media" backLabel="Back to media" />

{#if pageData.loading}
  <PageLoading message="Loading trash..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if (pageData.data?.media ?? []).length === 0}
  <EmptyState title="Trash is empty" description="Deleted media items will appear here." actionLabel="Back to Media Library" actionHref="/media" />
{:else}
  <p class="trash-info">
    Items in trash can be restored or permanently deleted. Permanently deleted items cannot be recovered.
  </p>

  <ListGrid minItemWidth={26}>
    {#each pageData.data?.media ?? [] as item}
      {@const accent = getMediaKindAccent(item.kind)}
      <ListCard
        title={item.title ?? item.originalFilename ?? "Untitled"}
        href={`/media/${item.id}`}
        accent="#64748b"
        actionsPlacement={item.thumbnailUrl ? "media-overlay" : "media"}
      >
        {#snippet media()}
          <MediaThumbnail
            thumbnailUrl={item.thumbnailUrl}
            kind={item.kind}
            alt={item.title ?? ""}
            size="fill"
          />
        {/snippet}

        {#snippet trailing()}
          <div class="media-pills">
            <Pill {accent}>
              {getMediaKindLabel(item.kind)}
            </Pill>
            <Pill accent="#ef4444">Deleted</Pill>
          </div>
        {/snippet}

        <span class="media-meta">
          {#if item.byteSize}
            {formatFileSize(item.byteSize)} &middot;
          {/if}
          {#if item.deletedAt}
            Deleted {new Date(item.deletedAt).toLocaleDateString()}
          {/if}
        </span>

        {#snippet actions({ trigger, align })}
          <div class="trash-actions">
            <Button type="button" variant="subtle" size="sm" onclick={() => handleRestore(item.id)}>
              <RotateCcw size={14} />
              Restore
            </Button>
            <ConfirmAction
              title="Permanently Delete"
              description={`Are you sure you want to permanently delete "${item.title ?? item.originalFilename}"? This action cannot be undone.`}
              confirmLabel="Delete Forever"
              triggerLabel="Delete"
              triggerVariant="danger"
              onConfirm={() => handlePurge(item.id)}
            />
          </div>
        {/snippet}
      </ListCard>
    {/each}
  </ListGrid>
{/if}

<style>
  .trash-info {
    margin: 0 0 1.5rem;
    padding: 0.75rem 1rem;
    background: var(--bg-warning, rgba(251, 191, 36, 0.1));
    border: 1px solid var(--border-warning, rgba(251, 191, 36, 0.3));
    border-radius: 0.5rem;
    color: var(--text-warning, #b45309);
    font-size: 0.875rem;
  }

  .media-pills {
    display: flex;
    gap: 0.25rem;
  }

  .media-meta {
    font-size: 0.875rem;
    color: var(--admin-color-text-muted, #9ca3af);
  }

  .trash-actions {
    display: flex;
    gap: 0.5rem;
  }
</style>
