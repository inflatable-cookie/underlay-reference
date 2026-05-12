<script lang="ts">
  import { useAuthenticatedData } from "@decodelabs/underlay/runtime/auth";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";
  import { EntityTrashPage } from "@decodelabs/underlay/templates";
  import { mediaCommands, type MediaSummary, type PagedListResponse } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { MediaTrashListCard } from "$lib/cards";

  interface Props {
    title?: string;
    backHref?: string;
    backLabel?: string;
  }

  let {
    title = "Media Trash",
    backHref = "/media",
    backLabel = "Back to media"
  }: Props = $props();

  const toastStore = useToasts();

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
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to restore media";
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
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to delete media";
      toastStore.push({ variant: "error", message });
    }
  }
</script>

{#snippet renderItem(item: MediaSummary)}
  <MediaTrashListCard
    media={item}
    onRestore={handleRestore}
    onPurge={handlePurge}
  />
{/snippet}

<EntityTrashPage
  title={title}
  backHref={backHref}
  backLabel={backLabel}
  loading={pageData.loading}
  loadingMessage="Loading trash..."
  error={pageData.error}
  statusMessage="Items in trash can be restored or permanently deleted. Permanently deleted items cannot be recovered."
  statusTone="warning"
  items={pageData.data?.media.data ?? []}
  renderItem={renderItem}
  emptyTitle="Trash is empty"
  emptyMessage="Deleted media items will appear here."
/>
