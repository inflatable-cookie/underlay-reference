<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { EntityListPage } from "@decodelabs/underlay/templates";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import {
    buildQueryString,
    parseQueryParams,
    type QueryParams
  } from "@decodelabs/underlay/client/query";
  import { copyToClipboard, useToasts } from "@decodelabs/underlay/runtime/feedback";
  import { IconButton } from "@poodle/svelte";
  import { MediaKind, MediaVisibility } from "@decodelabs/underlay/runtime/media";
  import { MediaListCard } from "$lib/cards";
  import { mediaCommands, type MediaSummary } from "@api-client";
  import { auth } from "$lib/stores/auth";

  const toastStore = useToasts();

  const currentQuery = $derived(parseQueryParams($page.url.searchParams));

  function updateUrl(nextQuery: QueryParams) {
    const url = new URL($page.url);
    url.search = buildQueryString(nextQuery);
    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  async function dataLoader(fetch: typeof window.fetch, token: string | null, query: QueryParams) {
    if (!token) throw new Error("Not authenticated");
    return await mediaCommands.listMedia(fetch, token, {
      profile: "list",
      query
    });
  }

  async function handleDeleteMedia(mediaId: string) {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    await mediaCommands.softDeleteMedia(mediaId, fetch, token);
    toastStore.push({ variant: "success", message: "Media moved to trash" });
  }

  async function handleBatchDelete(ids: string[]) {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    const result = await mediaCommands.batchDeleteMedia({ ids }, fetch, token);
    toastStore.push({
      variant: "success",
      message: `Moved ${result.deleted} ${result.deleted === 1 ? "item" : "items"} to trash`
    });
  }

  async function handleCopyId(mediaId: string) {
    await copyToClipboard(toastStore, mediaId, "Copied media ID");
  }

  function handleUpload() {
    void gotoWithContext("/media/upload", {
      label: "Media",
      href: "/media",
      type: "list"
    });
  }

  function handleViewTrash() {
    void goto("/media/trash");
  }

  const filters = [
    {
      id: "title",
      type: "search" as const,
      label: "Title",
      placeholder: "Search by title..."
    },
    {
      id: "kind",
      type: "select" as const,
      label: "Type",
      options: [
        { value: "All", label: "All types" },
        { value: MediaKind.Image, label: "Image" },
        { value: MediaKind.Pdf, label: "PDF" },
        { value: MediaKind.Document, label: "Document" },
        { value: MediaKind.Video, label: "Video" },
        { value: MediaKind.Audio, label: "Audio" }
      ]
    },
    {
      id: "visibility",
      type: "select" as const,
      label: "Visibility",
      options: [
        { value: "All", label: "All visibility" },
        { value: MediaVisibility.Public, label: "Public" },
        { value: MediaVisibility.Restricted, label: "Restricted" }
      ]
    },
    {
      id: "sort",
      type: "sort" as const,
      label: "Sort",
      sortFields: [
        { key: "title", label: "Title" },
        { key: "kind", label: "Kind" },
        { key: "updatedAt", label: "Updated", defaultDirection: "desc" as const },
        { key: "createdAt", label: "Created", defaultDirection: "desc" as const }
      ]
    }
  ];

  const batchActions = [
    {
      id: "delete",
      label: "Delete",
      tone: "danger" as const,
      icon: "trash-2",
      confirm: {
        title: "Move media to trash",
        description: (count: number) =>
          `Are you sure you want to move ${count} media ${count === 1 ? "item" : "items"} to trash?`,
        confirmLabel: "Move to trash",
        cancelLabel: "Keep media"
      },
      handler: handleBatchDelete
    }
  ];
</script>

{#snippet mediaHeaderActions(ctx: { selectionMode: boolean; reorderMode: boolean; visibleItemCount: number })}
  <IconButton
    type="button"
    variant="secondary"
    tone="danger"
    icon="trash-2"
    ariaLabel="View trash"
    tooltip="View Trash"
    disabled={ctx.selectionMode || ctx.reorderMode}
    on:click={handleViewTrash}
  />
{/snippet}

{#snippet mediaCard(media: MediaSummary, ctx: { selectionMode: boolean; reorderMode: boolean; selected: boolean; onToggle: (selected: boolean) => void; refetch: () => Promise<void> })}
  <MediaListCard
    {media}
    reorderMode={ctx.reorderMode}
    selectionMode={ctx.selectionMode}
    selected={ctx.selected}
    onSelectionChange={(id, selected) => ctx.onToggle(selected)}
    onDelete={ctx.selectionMode || ctx.reorderMode ? undefined : async (id) => {
      await handleDeleteMedia(id);
      await ctx.refetch();
    }}
    onCopyId={ctx.selectionMode || ctx.reorderMode ? undefined : handleCopyId}
  />
{/snippet}

<EntityListPage
  title="Media Library"
  backHref="/"
  backLabel="Back to dashboard"
  {dataLoader}
  presentation="cards"
  renderItem={mediaCard as never}
  {filters}
  query={currentQuery}
  {batchActions}
  onQueryChange={updateUrl}
  onAdd={handleUpload}
  addLabel="Upload media"
  headerLeadingActions={mediaHeaderActions as never}
/>
