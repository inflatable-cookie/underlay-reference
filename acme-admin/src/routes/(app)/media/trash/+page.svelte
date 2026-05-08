<script lang="ts">
import {
  getMediaKindLabel,
  getMediaKindAccent
} from "@decodelabs/underlay/runtime/media";
import {
  useToasts,
} from "@decodelabs/underlay/runtime/feedback";
import {
  useAuthenticatedData,
} from "@decodelabs/underlay/runtime/auth";
  import {
  EmptyState as PoodleEmptyState,
  MediaThumbnail as PoodleMediaThumbnail,
  PageHeader as PoodlePageHeader,
  PageLoading } from "@poodle/svelte";
  import { AlertDialog as PoodleAlertDialog,
  Callout as PoodleCallout,
  ListCard as PoodleListCard,
  ListGrid,
  formatFileSize } from "@poodle/svelte";
    import { Button as PoodleButton, Pill as PoodlePill } from "@poodle/svelte";
  import { mediaCommands, type MediaSummary, type PagedListResponse } from "@api-client";
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
      defaultValue: {
        media: {
          data: [],
          total: 0,
          hasMore: false
        } as PagedListResponse<MediaSummary>
      }
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

  let purgeCandidate = $state<MediaSummary | null>(null);

  function toPoodleMediaKind(kind: string): "image" | "audio" | "video" | "document" | "embed" {
    if (kind === "image") return "image";
    if (kind === "audio") return "audio";
    if (kind === "video") return "video";
    return "document";
  }

</script>

<PoodlePageHeader title="Media Trash" backHref="/media" backLabel="Back to media" />

{#if pageData.loading}
  <PageLoading presentation="inline" message="Loading trash..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if (pageData.data?.media.data ?? []).length === 0}
  <PoodleEmptyState title="Trash is empty" message="Deleted media items will appear here.">
    <svelte:fragment slot="visual">
      <Trash2 size={18} />
    </svelte:fragment>
    <a slot="actions" href="/media">Back to Media Library</a>
  </PoodleEmptyState>
{:else}
  <p class="trash-info">
    Items in trash can be restored or permanently deleted. Permanently deleted items cannot be recovered.
  </p>

  <ListGrid minItemWidth={26}>
    {#each pageData.data?.media.data ?? [] as item}
      <PoodleListCard
        title={item.title ?? item.originalFilename ?? "Untitled"}
        href={`/media/${item.id}`}
        accentColor="#64748b"
      >
        <svelte:fragment slot="leading">
          <PoodleMediaThumbnail
            kind={toPoodleMediaKind(item.kind)}
            presentation="compact"
            aspectRatio="square"
            ariaLabel={item.title ?? "Media thumbnail"}
          >
            {#if item.thumbnailUrl}
              <img
                src={item.thumbnailUrl}
                alt={item.title ?? ""}
                class="media-thumbnail-image"
              />
            {/if}
          </PoodleMediaThumbnail>
        </svelte:fragment>

        <svelte:fragment slot="trailing">
          <div class="media-pills">
            <PoodlePill tone="neutral" appearance="badge" size="lg">{getMediaKindLabel(item.kind)}</PoodlePill>
            <PoodlePill tone="danger" appearance="badge" size="lg">Deleted</PoodlePill>
          </div>
        </svelte:fragment>

        <span slot="footer" class="media-meta">
          {#if item.byteSize}
            {formatFileSize(item.byteSize)} &middot;
          {/if}
          {#if item.deletedAt}
            Deleted {new Date(item.deletedAt).toLocaleDateString()}
          {/if}
        </span>

        <div slot="actions" class="trash-actions">
          <PoodleButton type="button" variant="ghost" size="sm" on:click={() => handleRestore(item.id)}>
            <RotateCcw size={14} />
            Restore
          </PoodleButton>
          <PoodleButton
            type="button"
            variant="ghost"
            tone="danger"
            size="sm"
            on:click={() => (purgeCandidate = item)}
          >
            <Trash2 size={14} />
            Delete
          </PoodleButton>
        </div>
      </PoodleListCard>
    {/each}
  </ListGrid>
{/if}

<PoodleAlertDialog
  open={purgeCandidate !== null}
  title="Permanently Delete"
  description={purgeCandidate
    ? `Are you sure you want to permanently delete "${purgeCandidate.title ?? purgeCandidate.originalFilename}"? This action cannot be undone.`
    : null}
  confirmLabel="Delete Forever"
  tone="danger"
  onConfirm={async () => {
    if (!purgeCandidate) return;
    const mediaId = purgeCandidate.id;
    purgeCandidate = null;
    await handlePurge(mediaId);
  }}
  onCancel={() => {
    purgeCandidate = null;
  }}
/>

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

  :global(.media-thumbnail-image) {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
</style>
