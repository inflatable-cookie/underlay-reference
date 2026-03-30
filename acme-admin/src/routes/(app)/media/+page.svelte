<script lang="ts">
import {
  useToasts,
  useAuthenticatedData,
  useBatchSelection,
  getMediaKindAccent,
  getMediaKindLabel,
  getMediaKindIcon,
  getMediaVisibilityAccent,
  getMediaVisibilityLabel,
  formatFileSize,
  MediaKind,
  MediaVisibility
} from "@decodelabs/underlay/runtime";
import {
  EmptyState as PoodleEmptyState,
  FilterToolbar,
  MediaThumbnail as PoodleMediaThumbnail,
  PageHeader as PoodlePageHeader,
  PageLoading } from "@poodle/svelte-composites";
  import { AlertDialog as PoodleAlertDialog,
  Callout as PoodleCallout,
  Grid as PoodleGrid,
  ListCard as PoodleListCard,
  OrderBy as PoodleOrderBy,
  type BulkAction,
  type OrderByValue } from "@poodle/svelte-primitives";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
    import {
    BulkActionBar as PoodleBulkActionBar,
    Button as PoodleButton,
    Field as PoodleField,
    IconButton as PoodleIconButton,
    Pill as PoodlePill,
    SearchField as PoodleSearchField,
    Select as PoodleSelect
  } from "@poodle/svelte-primitives";
  import { gotoWithContext, parseQueryParams } from "@decodelabs/underlay/client";
  import { mediaCommands, type MediaSummary } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import CopyActionsMenu from "$lib/components/CopyActionsMenu.svelte";
  import { squareCheckIcon, trash2Icon, uploadIcon } from "$lib/ui/poodle-icon-nodes";
  import Upload from "lucide-svelte/icons/upload";
  import Image from "lucide-svelte/icons/image";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import CheckSquare from "lucide-svelte/icons/check-square";

  const toastStore = useToasts();

  // Track URL for refetching when filters change
  let previousUrl = $state<string | null>(null);

  // Fetch media using authenticated data pattern
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const query = parseQueryParams($page.url.searchParams);
      const response = await mediaCommands.listMedia(fetch, token, {
        profile: "list",
        query,
      });
      return { items: response.data };
    },
    {
      defaultValue: { items: [] as MediaSummary[] },
      onSuccess: () => {
        previousUrl = $page.url.search;
      }
    }
  );

  // Refetch when URL changes (for sorting/filtering)
  $effect(() => {
    const currentUrl = $page.url.search;
    if (previousUrl !== null && previousUrl !== currentUrl) {
      previousUrl = currentUrl;
      pageData.refetch();
    }
  });

  // Parse current state from URL
  const currentQuery = $derived(parseQueryParams($page.url.searchParams));

  // Convert URL sort to OrderByValue format
  const orderBy: OrderByValue = $derived(
    (currentQuery.sort ?? []).map((s) => ({ key: s.field, direction: s.direction }))
  );

  // Get filter values from URL
  const selectedTitle = $derived(
    currentQuery.filters?.find((f) => f.field === "title")?.value ?? ""
  );
  const selectedKind = $derived(
    currentQuery.filters?.find((f) => f.field === "kind")?.value ?? "All"
  );
  const selectedVisibility = $derived(
    currentQuery.filters?.find((f) => f.field === "visibility")?.value ?? "All"
  );

  const sortFields = [
    { key: "title", label: "Title" },
    { key: "kind", label: "Kind" },
    { key: "updatedAt", label: "Updated", defaultDirection: "desc" as const },
    { key: "createdAt", label: "Created", defaultDirection: "desc" as const }
  ];

  const kindItems = [
    { value: "All", label: "All types" },
    { value: MediaKind.Image, label: "Image" },
    { value: MediaKind.Pdf, label: "PDF" },
    { value: MediaKind.Document, label: "Document" },
    { value: MediaKind.Video, label: "Video" },
    { value: MediaKind.Audio, label: "Audio" }
  ];

  const visibilityItems = [
    { value: "All", label: "All visibility" },
    { value: "public", label: "Public" },
    { value: "restricted", label: "Restricted" }
  ];

  function toPoodleMediaKind(kind: string): "image" | "audio" | "video" | "document" | "embed" {
    if (kind === MediaKind.Image) return "image";
    if (kind === MediaKind.Audio) return "audio";
    if (kind === MediaKind.Video) return "video";
    return "document";
  }

  let titleFilterInput = $state("");
  let titleFilterTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    titleFilterInput = selectedTitle.replace(/%/g, "");
  });

  // Update URL when sort changes
  function handleSortChange(newOrderBy: OrderByValue) {
    const url = new URL($page.url);

    if (newOrderBy.length > 0) {
      const sortString = newOrderBy.map((s) => `${s.key}:${s.direction}`).join(",");
      url.searchParams.set("sort", sortString);
    } else {
      url.searchParams.delete("sort");
    }

    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  // Update URL when title filter changes
  function handleTitleChange(title: string) {
    const url = new URL($page.url);

    if (title && title.trim()) {
      url.searchParams.set("filter[title][like]", `%${title.trim()}%`);
    } else {
      url.searchParams.delete("filter[title][like]");
    }

    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  function handleTitleInput(value: string) {
    titleFilterInput = value;
    if (titleFilterTimer) clearTimeout(titleFilterTimer);
    titleFilterTimer = setTimeout(() => {
      handleTitleChange(value);
    }, 500);
  }

  // Update URL when kind filter changes
  function handleKindChange(value: string) {
    const url = new URL($page.url);

    if (value && value !== "All") {
      url.searchParams.set("filter[kind]", value);
    } else {
      url.searchParams.delete("filter[kind]");
    }

    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  // Update URL when visibility filter changes
  function handleVisibilityChange(value: string) {
    const url = new URL($page.url);

    if (value && value !== "All") {
      url.searchParams.set("filter[visibility]", value);
    } else {
      url.searchParams.delete("filter[visibility]");
    }

    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  async function handleDeleteMedia(mediaId: string) {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await mediaCommands.softDeleteMedia(mediaId, fetch, token);
      toastStore.push({ variant: "success", message: "Media moved to trash" });
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete media";
      toastStore.push({ variant: "error", message });
    }
  }

  // Selection mode state
  let isSelectionMode = $state(false);
  const selection = useBatchSelection<string>();
  let batchLoading = $state(false);
  let showBatchDeleteConfirm = $state(false);

  // Clear selection when exiting selection mode
  $effect(() => {
    if (!isSelectionMode) {
      selection.clear();
    }
  });

  function toggleSelectionMode() {
    isSelectionMode = !isSelectionMode;
  }

  function handleClearSelection() {
    selection.clear();
    isSelectionMode = false;
  }

  function handleSelectAll() {
    const allIds = (pageData.data?.items ?? []).map(m => m.id);
    if (selection.count === allIds.length) {
      handleClearSelection();
      return;
    }
    selection.selectAll(allIds);
  }

  async function handleBatchDelete() {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    showBatchDeleteConfirm = false;
    batchLoading = true;
    try {
      const result = await mediaCommands.batchDeleteMedia(
        { ids: selection.selectedIds },
        fetch,
        token
      );
      toastStore.push({
        variant: "success",
        message: `Moved ${result.deleted} ${result.deleted === 1 ? "item" : "items"} to trash`
      });
      selection.clear();
      isSelectionMode = false;
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete media";
      toastStore.push({ variant: "error", message });
    } finally {
      batchLoading = false;
    }
  }

  const batchActions: BulkAction[] = [
    { id: "delete", label: "Delete", icon: trash2Icon, tone: "danger" }
  ];
</script>

<PoodlePageHeader title="Media Library" backHref="/" backLabel="Back to dashboard">
  <svelte:fragment slot="actions">
    {#if (pageData.data?.items ?? []).length > 0}
      <PoodleIconButton
        type="button"
        variant="secondary"
        tone={isSelectionMode ? "danger" : "default"}
        icon={squareCheckIcon}
        ariaLabel={isSelectionMode ? "Cancel selection" : "Select items"}
        tooltip={isSelectionMode ? "Cancel Selection" : "Select Items"}
        on:click={toggleSelectionMode}
      />
    {/if}
    {#if !isSelectionMode}
      <PoodleIconButton
        type="button"
        variant="secondary"
        tone="danger"
        icon={trash2Icon}
        ariaLabel="View trash"
        tooltip="View Trash"
        on:click={() => goto("/media/trash")}
      />
    {/if}
    {#if !isSelectionMode}
      <PoodleIconButton
        type="button"
        variant="primary"
        icon={uploadIcon}
        ariaLabel="Upload media"
        tooltip="Upload Media"
        on:click={() =>
          void gotoWithContext("/media/upload", {
            label: "Media",
            href: "/media",
            type: "list"
          })}
      />
    {/if}
  </svelte:fragment>
</PoodlePageHeader>

<FilterToolbar ariaLabel="Media filters" summaryText="Filters">
  <svelte:fragment slot="actions">
    <PoodleButton type="button" variant="ghost" size="sm" onclick={() => pageData.refetch()}>
      Refresh
    </PoodleButton>
  </svelte:fragment>
  <PoodleField id="media-filter-title" label="Title" let:describedBy>
    <PoodleSearchField
      id="media-filter-title"
      value={titleFilterInput}
      describedBy={describedBy}
      placeholder="Search by title..."
      on:valueChange={(event) => handleTitleInput(event.detail.value)}
    />
  </PoodleField>
  <PoodleField id="media-filter-kind" label="Type" let:describedBy>
    <PoodleSelect
      id="media-filter-kind"
      value={selectedKind}
      describedBy={describedBy}
      options={kindItems}
      placeholder="All types"
      on:valueChange={(event) => handleKindChange(event.detail.value)}
    />
  </PoodleField>
  <PoodleField id="media-filter-visibility" label="Visibility" let:describedBy>
    <PoodleSelect
      id="media-filter-visibility"
      value={selectedVisibility}
      describedBy={describedBy}
      options={visibilityItems}
      placeholder="All visibility"
      on:valueChange={(event) => handleVisibilityChange(event.detail.value)}
    />
  </PoodleField>
  <PoodleField id="media-filter-sort" label="Sort">
    <PoodleOrderBy fields={sortFields} value={orderBy} onChange={handleSortChange} compact />
  </PoodleField>
</FilterToolbar>

{#if pageData.loading}
  <PageLoading presentation="inline" message="Loading media..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if (pageData.data?.items ?? []).length === 0}
  <PoodleEmptyState title="No media found">
    <svelte:fragment slot="visual">
      <Image size={18} />
    </svelte:fragment>
    <a slot="actions" href="/media/upload">Upload your first media</a>
  </PoodleEmptyState>
{:else}
  <PoodleGrid columns="repeat(auto-fit, minmax(min(26em, 100%), 1fr))" gap="lg">
    {#each pageData.data?.items ?? [] as item}
      {@const accent = getMediaKindAccent(item.kind)}
      <PoodleListCard
        title={item.title ?? item.originalFilename ?? "Untitled"}
        subtitle={isSelectionMode ? formatFileSize(item.byteSize) : undefined}
        href={isSelectionMode ? undefined : `/media/${item.id}`}
        accentColor={accent}
        selectable={isSelectionMode}
        selected={selection.isSelected(item.id)}
        on:selectedChange={(event) => {
          if (isSelectionMode) {
            selection.toggle(item.id, event.detail.selected);
          }
        }}
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
            {#if item.visibility && item.visibility !== MediaVisibility.Public}
              <PoodlePill tone="neutral" appearance="badge" size="lg">
                {getMediaVisibilityLabel(item.visibility)}
              </PoodlePill>
            {/if}
          </div>
        </svelte:fragment>
        <svelte:fragment slot="actions">
          {#if !isSelectionMode}
            <CopyActionsMenu
              toastStore={toastStore}
              triggerLabel="Actions"
              copies={[
                {
                  label: "Copy media ID",
                  text: item.id,
                  successMessage: "Copied media ID"
                }
              ]}
              actions={[
                {
                  label: "View details",
                  onSelect: () =>
                    void gotoWithContext(`/media/${item.id}`, {
                      label: "Media",
                      href: "/media",
                      type: "list"
                    })
                },
                {
                  label: "Move to trash",
                  destructive: true,
                  onSelect: () => handleDeleteMedia(item.id)
                }
              ]}
            />
          {/if}
        </svelte:fragment>

        <span slot="footer" class="media-meta">
          {#if item.byteSize}
            {formatFileSize(item.byteSize)} &middot;
          {/if}
          Updated {new Date(item.updatedAt).toLocaleDateString()}
        </span>
      </PoodleListCard>
    {/each}
  </PoodleGrid>
{/if}

<PoodleBulkActionBar
  selectionCount={selection.count}
  totalCount={(pageData.data?.items ?? []).length}
  actions={batchActions}
  loading={batchLoading}
  showSelectAll
  allSelected={selection.count > 0 && selection.count === (pageData.data?.items ?? []).length}
  on:clear={handleClearSelection}
  on:selectAll={handleSelectAll}
  on:action={() => (showBatchDeleteConfirm = true)}
/>

<PoodleAlertDialog
  open={showBatchDeleteConfirm}
  title="Delete selected media"
  description={`Are you sure you want to delete ${selection.count} selected ${selection.count === 1 ? "item" : "items"}?`}
  confirmLabel={`Delete ${selection.count} ${selection.count === 1 ? "item" : "items"}`}
  tone="danger"
  onConfirm={handleBatchDelete}
  onCancel={() => {
    showBatchDeleteConfirm = false;
  }}
/>

<style>
  .media-pills {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    justify-content: flex-end;
  }

  .media-meta {
    color: var(--admin-color-text-muted);
  }

  :global(.media-thumbnail-image) {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
</style>
