<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import {
    FilterBar,
    PageHeader,
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
    Button,
    EmptyState,
    Field,
    FormError,
    ListGrid,
    ListCard,
    MediaThumbnail,
    Pill,
    OrderBy,
    PageLoading,
    Select,
    TextInput,
    Tooltip,
    type OrderByValue
  } from "@decodelabs/underlay/components";
  import { gotoWithContext, parseQueryParams } from "@decodelabs/underlay/client";
  import { mediaCommands, type MediaSummary } from "acme-client";
  import { BatchActionBar } from "$lib/components";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
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
      const items = await mediaCommands.listMediaAdmin(fetch, token, query);
      return { items };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { items: [] as MediaSummary[] },
      onSuccess: () => {
        previousUrl = $page.url.search;
      }
    }
  );

  // Trigger fetch when auth is ready
  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

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

<PageHeader section="Media Library" backHref="/" backLabel="Back to dashboard">
  {#snippet actions()}
    {#if (pageData.data?.items ?? []).length > 0}
      <Tooltip content={isSelectionMode ? "Cancel Selection" : "Select Items"} inline>
        {#snippet trigger()}
          <Button
            type="button"
            variant={isSelectionMode ? "danger" : "subtle"}
            size="icon"
            onclick={toggleSelectionMode}
          >
            <CheckSquare size={16} />
          </Button>
        {/snippet}
      </Tooltip>
    {/if}
    {#if !isSelectionMode}
      <Tooltip content="View Trash" inline>
        {#snippet trigger()}
          <Button type="button" variant="danger-subtle" size="icon" onclick={() => goto("/media/trash")}>
            <Trash2 size={16} />
          </Button>
        {/snippet}
      </Tooltip>
    {/if}
    {#if !isSelectionMode}
      <Tooltip content="Upload Media" inline>
        {#snippet trigger()}
          <Button
            type="button"
            variant="primary"
            size="icon"
            onclick={() =>
              void gotoWithContext("/media/upload", {
                label: "Media",
                href: "/media",
                type: "list"
              })}
          >
            <Upload size={16} />
          </Button>
        {/snippet}
      </Tooltip>
    {/if}
  {/snippet}
</PageHeader>

<FilterBar title="Filters" onRefresh={() => pageData.refetch()}>
  <Field label="Title" forId="title">
    <TextInput
      id="title"
      value={selectedTitle.replace(/%/g, "")}
      onchange={handleTitleChange}
      debounce={500}
      search
      placeholder="Search by title..."
    />
  </Field>
  <Field label="Type" forId="kind">
    <Select
      id="kind"
      value={selectedKind}
      onchange={handleKindChange}
      items={kindItems}
      placeholder="All types"
      clearable
      defaultValue="All"
    />
  </Field>
  <Field label="Visibility" forId="visibility">
    <Select
      id="visibility"
      value={selectedVisibility}
      onchange={handleVisibilityChange}
      items={visibilityItems}
      placeholder="All visibility"
      clearable
      defaultValue="All"
    />
  </Field>
  <Field label="Sort" forId="sort">
    <OrderBy fields={sortFields} value={orderBy} onChange={handleSortChange} />
  </Field>
</FilterBar>

{#if pageData.loading}
  <PageLoading message="Loading media..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
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
          <MediaThumbnail
            thumbnailUrl={item.thumbnailUrl}
            kind={item.kind}
            alt={item.title ?? ""}
            size="fill"
          />
        {/snippet}
        {#snippet trailing()}
          <div class="media-pills">
            <Pill accent={accent}>
              {getMediaKindLabel(item.kind)}
            </Pill>
            {#if item.visibility && item.visibility !== MediaVisibility.Public}
              <Pill accent={getMediaVisibilityAccent(item.visibility)}>
                {getMediaVisibilityLabel(item.visibility)}
              </Pill>
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
</style>
