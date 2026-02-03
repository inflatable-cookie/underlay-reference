<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import {
    FilterBar,
    PageHeader,
    useToasts,
    useAuthenticatedData,
    getMediaKindLabel,
    MediaKind
  } from "@decodelabs/underlay/patterns";
  import {
    Button,
    Field,
    FormError,
    ListGrid,
    ListCard,
    Badge,
    OrderBy,
    PageLoading,
    Select,
    TextInput,
    type OrderByValue
  } from "@decodelabs/underlay/components";
  import { gotoWithContext, parseQueryParams } from "@decodelabs/underlay/client";
  import { mediaCommands, type MediaSummary } from "acme-client";
  import { BatchActionBar } from "$lib/components";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import Plus from "lucide-svelte/icons/plus";
  import Image from "lucide-svelte/icons/image";
  import FileText from "lucide-svelte/icons/file-text";
  import Film from "lucide-svelte/icons/film";
  import Music from "lucide-svelte/icons/music";
  import FileIcon from "lucide-svelte/icons/file";
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

  function getKindIcon(kind: string) {
    switch (kind) {
      case MediaKind.Image:
        return Image;
      case MediaKind.Video:
        return Film;
      case MediaKind.Audio:
        return Music;
      case MediaKind.Document:
      case MediaKind.Pdf:
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
      case MediaKind.Pdf:
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

  // Selection mode state
  let isSelectionMode = $state(false);
  let selectedIds = $state<Set<string>>(new Set());
  let batchLoading = $state(false);

  // Clear selection when exiting selection mode
  $effect(() => {
    if (!isSelectionMode) {
      selectedIds = new Set();
    }
  });

  function toggleSelectionMode() {
    isSelectionMode = !isSelectionMode;
    if (!isSelectionMode) {
      selectedIds = new Set();
    }
  }

  function handleSelectionChange(mediaId: string, selected: boolean) {
    const newSet = new Set(selectedIds);
    if (selected) {
      newSet.add(mediaId);
    } else {
      newSet.delete(mediaId);
    }
    selectedIds = newSet;
  }

  function clearSelection() {
    selectedIds = new Set();
    isSelectionMode = false;
  }

  function selectAll() {
    const allIds = (pageData.data?.items ?? []).map(m => m.id);
    selectedIds = new Set(allIds);
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
        { ids: Array.from(selectedIds) },
        fetch,
        token
      );
      toastStore.push({
        variant: "success",
        message: `Moved ${result.deleted} ${result.deleted === 1 ? "item" : "items"} to trash`
      });
      selectedIds = new Set();
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

<PageHeader title="Media Library" backHref="/" backLabel="Back to dashboard">
  {#snippet actions()}
    {#if !isSelectionMode}
      <Button
        type="button"
        variant="subtle"
        onclick={() => goto("/media/trash")}
      >
        <Trash2 size={16} />
        Trash
      </Button>
    {/if}
    {#if (pageData.data?.items ?? []).length > 0}
      <Button
        type="button"
        variant={isSelectionMode ? "danger" : "subtle"}
        onclick={toggleSelectionMode}
      >
        <CheckSquare size={16} />
        {isSelectionMode ? "Cancel" : "Select"}
      </Button>
    {/if}
    {#if !isSelectionMode}
      <Button
        type="button"
        variant="primary"
        onclick={() =>
          void gotoWithContext("/media/upload", {
            label: "Media",
            href: "/media",
            type: "list"
          })}
      >
        <Plus size={16} />
        Upload
      </Button>
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
  <p class="empty-state">No media found. Upload your first file to get started.</p>
{:else}
  <ListGrid minItemWidth={20}>
    {#each pageData.data?.items ?? [] as item}
      {@const KindIcon = getKindIcon(item.kind)}
      {#if isSelectionMode}
        <div class="selectable-card">
          <label class="selection-checkbox">
            <input
              type="checkbox"
              checked={selectedIds.has(item.id)}
              onchange={(e) => handleSelectionChange(item.id, e.currentTarget.checked)}
            />
          </label>
          <ListCard
            title={item.title ?? item.originalFilename ?? "Untitled"}
            subtitle={formatFileSize(item.byteSize)}
            onclick={() => handleSelectionChange(item.id, !selectedIds.has(item.id))}
          >
            {#snippet media()}
              <KindIcon size={20} />
            {/snippet}
            {#snippet titleSuffix()}
              <Badge variant={getKindVariant(item.kind)} size="sm">
                {getMediaKindLabel(item.kind)}
              </Badge>
            {/snippet}
          </ListCard>
        </div>
      {:else}
        <ListCard
          title={item.title ?? item.originalFilename ?? "Untitled"}
          subtitle={formatFileSize(item.byteSize)}
          href={`/media/${item.id}`}
        >
          {#snippet media()}
            <KindIcon size={20} />
          {/snippet}
          {#snippet titleSuffix()}
            <Badge variant={getKindVariant(item.kind)} size="sm">
              {getMediaKindLabel(item.kind)}
            </Badge>
          {/snippet}
        </ListCard>
      {/if}
    {/each}
  </ListGrid>
{/if}

<BatchActionBar
  selectedCount={selectedIds.size}
  totalCount={(pageData.data?.items ?? []).length}
  loading={batchLoading}
  onClearSelection={clearSelection}
  onSelectAll={selectAll}
  onBatchDelete={handleBatchDelete}
/>

<style>
  .empty-state {
    padding: 2rem;
    text-align: center;
    color: var(--text-secondary, #6b7280);
  }

  .selectable-card {
    position: relative;
  }

  .selection-checkbox {
    position: absolute;
    top: 0.5rem;
    left: 0.5rem;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.5rem;
    height: 1.5rem;
    background: var(--surface-elevated, #1f2937);
    border-radius: 0.25rem;
    cursor: pointer;
  }

  .selection-checkbox input {
    width: 1rem;
    height: 1rem;
    accent-color: var(--accent-color, #3b82f6);
    cursor: pointer;
  }
</style>
