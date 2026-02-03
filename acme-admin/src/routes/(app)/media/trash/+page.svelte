<script lang="ts">
  import { goto } from "$app/navigation";
  import {
    PageHeader,
    useToasts,
    useAuthenticatedData,
    getMediaKindLabel,
    MediaKind
  } from "@decodelabs/underlay/patterns";
  import {
    Button,
    FormError,
    ListGrid,
    ListCard,
    Badge,
    PageLoading,
    ConfirmAction
  } from "@decodelabs/underlay/components";
  import { mediaCommands, type MediaSummary } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import RotateCcw from "lucide-svelte/icons/rotate-ccw";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import Image from "lucide-svelte/icons/image";
  import FileText from "lucide-svelte/icons/file-text";
  import Film from "lucide-svelte/icons/film";
  import Music from "lucide-svelte/icons/music";
  import FileIcon from "lucide-svelte/icons/file";

  const toastStore = useToasts();

  // Fetch trashed media
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const media = await mediaCommands.listMediaTrash(fetch, token);
      return { media };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { media: [] as MediaSummary[] }
    }
  );

  // Trigger fetch when auth is ready
  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

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

  function getKindIcon(kind: string) {
    switch (kind) {
      case MediaKind.Image:
        return Image;
      case MediaKind.Video:
        return Film;
      case MediaKind.Audio:
        return Music;
      case MediaKind.Document:
        return FileText;
      default:
        return FileIcon;
    }
  }

  function getKindVariant(kind: string): "default" | "success" | "warning" | "danger" | "info" | "muted" {
    switch (kind) {
      case MediaKind.Image:
        return "info";
      case MediaKind.Video:
        return "warning";
      case MediaKind.Audio:
        return "success";
      case MediaKind.Document:
        return "muted";
      default:
        return "default";
    }
  }

  function formatFileSize(bytes: number | null | undefined): string {
    if (bytes == null || bytes === 0) return "—";
    const units = ["B", "KB", "MB", "GB"];
    let size = bytes;
    let unitIndex = 0;
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    return `${size.toFixed(unitIndex > 0 ? 1 : 0)} ${units[unitIndex]}`;
  }
</script>

<PageHeader title="Trash" backHref="/media" backLabel="Back to media" />

{#if pageData.loading}
  <PageLoading message="Loading trash..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if (pageData.data?.media ?? []).length === 0}
  <div class="empty-state">
    <Trash2 size={48} />
    <h2>Trash is empty</h2>
    <p>Deleted media items will appear here.</p>
    <Button type="button" variant="subtle" onclick={() => goto("/media")}>
      Back to Media Library
    </Button>
  </div>
{:else}
  <p class="trash-info">
    Items in trash can be restored or permanently deleted. Permanently deleted items cannot be recovered.
  </p>

  <ListGrid minItemWidth={20}>
    {#each pageData.data?.media ?? [] as item}
      {@const KindIcon = getKindIcon(item.kind)}
      <ListCard
        title={item.title ?? item.originalFilename ?? "Untitled"}
        subtitle={formatFileSize(item.byteSize)}
      >
        {#snippet media()}
          <KindIcon size={20} />
        {/snippet}
        {#snippet titleSuffix()}
          <Badge variant={getKindVariant(item.kind)} size="sm">
            {getMediaKindLabel(item.kind)}
          </Badge>
        {/snippet}
        {#snippet actions()}
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
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 2rem;
    text-align: center;
    color: var(--text-secondary, #6b7280);
  }

  .empty-state h2 {
    margin: 1rem 0 0.5rem;
    font-size: 1.25rem;
    color: var(--text-primary, #111827);
  }

  .empty-state p {
    margin: 0 0 1.5rem;
  }

  .trash-info {
    margin: 0 0 1.5rem;
    padding: 0.75rem 1rem;
    background: var(--bg-warning, rgba(251, 191, 36, 0.1));
    border: 1px solid var(--border-warning, rgba(251, 191, 36, 0.3));
    border-radius: 0.5rem;
    color: var(--text-warning, #b45309);
    font-size: 0.875rem;
  }

  .trash-actions {
    display: flex;
    gap: 0.5rem;
  }
</style>
