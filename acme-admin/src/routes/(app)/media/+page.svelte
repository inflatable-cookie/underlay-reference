<script lang="ts">
  import { MediaThumbnail as PoodleMediaThumbnail, PageHeader as PoodlePageHeader } from "@poodle/svelte-composites";
  import { Callout as PoodleCallout } from "@poodle/svelte-primitives";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import {
    FilterBar,
    CopyActionsMenu,
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
  } from "@decodelabs/underlay/patterns";
  import {
    BatchActionBar,
    EmptyState,
    ListGrid,
    ListCard,
    OrderBy,
    PageLoading,
    type OrderByValue
  } from "@decodelabs/underlay/components";
  import {
    Button as PoodleButton,
    Field as PoodleField,
    IconButton as PoodleIconButton,
    Pill as PoodlePill,
    SearchField as PoodleSearchField,
    Select as PoodleSelect
  } from "@poodle/svelte-primitives";
  import { gotoWithContext, parseQueryParams } from "@decodelabs/underlay/client";
  import { mediaCommands, type MediaSummary } from "acme-client";
  import { auth } from "$lib/stores/auth";
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
    selection.selectAll(allIds);
  }

  async function handleBatchDelete() {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

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

<FilterBar title="Filters" onRefresh={() => pageData.refetch()}>
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
    <OrderBy fields={sortFields} value={orderBy} onChange={handleSortChange} />
  </PoodleField>
</FilterBar>

{#if pageData.loading}
  <PageLoading message="Loading media..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if (pageData.data?.items ?? []).length === 0}
  <EmptyState title="No media found" actionLabel="Upload your first media" actionHref="/media/upload" />
{:else}
  <ListGrid minItemWidth={26}>
    {#each pageData.data?.items ?? [] as item}
      {@const accent = getMediaKindAccent(item.kind)}
      <ListCard
        title={item.title ?? item.originalFilename ?? "Untitled"}
        subtitle={isSelectionMode ? formatFileSize(item.byteSize) : undefined}
        href={isSelectionMode ? undefined : `/media/${item.id}`}
        {accent}
        selected={selection.isSelected(item.id)}
        onSelectionChange={isSelectionMode ? (checked) => selection.toggle(item.id, checked) : undefined}
        actionsPlacement={item.thumbnailUrl ? "media-overlay" : "media"}
      >
        {#snippet media()}
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
        {/snippet}
        {#snippet trailing()}
          <div class="media-pills">
            <PoodlePill tone="neutral" appearance="badge" size="lg">{getMediaKindLabel(item.kind)}</PoodlePill>
            {#if item.visibility && item.visibility !== MediaVisibility.Public}
              <PoodlePill tone="neutral" appearance="badge" size="lg">
                {getMediaVisibilityLabel(item.visibility)}
              </PoodlePill>
            {/if}
          </div>
        {/snippet}

        {#snippet actions({ trigger, align })}
          <CopyActionsMenu
            toastStore={toastStore}
            {trigger}
            {align}
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
        {/snippet}

        <span class="media-meta">
          {#if item.byteSize}
            {formatFileSize(item.byteSize)} &middot;
          {/if}
          Updated {new Date(item.updatedAt).toLocaleDateString()}
        </span>
      </ListCard>
    {/each}
  </ListGrid>
{/if}

<BatchActionBar
  selectedCount={selection.count}
  totalCount={(pageData.data?.items ?? []).length}
  loading={batchLoading}
  onClearSelection={handleClearSelection}
  onSelectAll={handleSelectAll}
  onBatchDelete={handleBatchDelete}
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
