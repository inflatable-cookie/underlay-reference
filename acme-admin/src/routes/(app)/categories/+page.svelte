<script lang="ts">
import {
  createReorderController,
} from "@decodelabs/underlay/runtime/data";
import {
  useToasts,
} from "@decodelabs/underlay/runtime/feedback";
import {
  useAuthenticatedData
} from "@decodelabs/underlay/runtime/auth";
import {
  EmptyState as PoodleEmptyState,
  FilterToolbar,
  PageHeader as PoodlePageHeader,
  PageLoading,
  EditableList as PoodleReorderableList } from "@poodle/svelte";
  import { Callout as PoodleCallout,
  Grid as PoodleGrid,
  ListCard as PoodleListCard,
  OrderBy as PoodleOrderBy,
  type OrderByValue } from "@poodle/svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
    import {
    Button as PoodleButton,
    Field as PoodleField,
    IconButton as PoodleIconButton,
    TextInput as PoodleSearchField,
    Select as PoodleSelect
  } from "@poodle/svelte";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import { parseQueryParams } from "@decodelabs/underlay/client/query";
  import { CategoryListCard } from "$lib/cards";
  import { recoverReorderConflict } from "$lib/lists/reorder-conflicts";
  import { arrowUpDownIcon } from "$lib/ui/poodle-icon-nodes";
  import { adminCommands, type CategoryWithCounts } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import ArrowUpDown from "lucide-svelte/icons/arrow-up-down";
  import Plus from "lucide-svelte/icons/plus";
  import FolderOpen from "lucide-svelte/icons/folder-open";

  const toastStore = useToasts();

  // Track URL for refetching when filters change
  let previousUrl = $state<string | null>(null);

  // Fetch categories using authenticated data pattern
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const query = parseQueryParams($page.url.searchParams);
      const categories = await adminCommands.listCategories(fetch, token, query);
      return { categories };
    },
    {
      defaultValue: { categories: [] as CategoryWithCounts[] },
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

  let isReorderMode = $state(false);
  let reorderSubmitError = $state<string | null>(null);

  // Parse current state from URL
  const currentQuery = $derived(parseQueryParams($page.url.searchParams));

  // Convert URL sort to OrderByValue format
  const orderBy: OrderByValue = $derived(
    (currentQuery.sort ?? []).map((s) => ({ key: s.field, direction: s.direction }))
  );

  // Get filter values from URL
  const selectedName = $derived(
    currentQuery.filters?.find((f) => f.field === "name")?.value ?? ""
  );
  const selectedIsActive = $derived(
    currentQuery.filters?.find((f) => f.field === "isActive")?.value ?? "All"
  );

  const sortFields = [
    { key: "name", label: "Name" },
    { key: "weight", label: "Weight" },
    { key: "projectCount", label: "Projects" },
    { key: "createdAt", label: "Created", defaultDirection: "desc" as const }
  ];

  const activeItems = [
    { value: "All", label: "All" },
    { value: "true", label: "Active" },
    { value: "false", label: "Inactive" }
  ];

  let nameFilterInput = $state("");
  let nameFilterTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    nameFilterInput = selectedName.replace(/%/g, "");
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

  // Update URL when name filter changes
  function handleNameChange(name: string) {
    const url = new URL($page.url);

    if (name && name.trim()) {
      url.searchParams.set("filter[name][like]", `%${name.trim()}%`);
    } else {
      url.searchParams.delete("filter[name][like]");
    }

    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  function handleNameInput(value: string) {
    nameFilterInput = value;

    if (nameFilterTimer) clearTimeout(nameFilterTimer);
    nameFilterTimer = setTimeout(() => {
      handleNameChange(value);
    }, 500);
  }

  // Update URL when active filter changes
  function handleActiveChange(value: string) {
    const url = new URL($page.url);

    if (value && value !== "All") {
      url.searchParams.set("filter[isActive]", value);
    } else {
      url.searchParams.delete("filter[isActive]");
    }

    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  // Map categories to have 'id' field for reorder controller
  const reorderItems = $derived(
    (pageData.data?.categories ?? []).map((c) => ({ ...c, id: c.id }))
  );

  // Create reorder controller
  const reorderController = $derived(
    createReorderController(reorderItems, async (orderedIds) => {
      const token = auth.getToken();
      if (!token) {
        toastStore.push({ variant: "error", message: "Not authenticated" });
        return;
      }
      await adminCommands.reorderCategories({ ids: orderedIds }, fetch, token);
    })
  );

  function enterReorderMode() {
    reorderSubmitError = null;
    isReorderMode = true;
  }

  async function handleReorderSuccess() {
    reorderSubmitError = null;
    isReorderMode = false;
    await pageData.refetch();
  }

  async function handleReorderError(error: unknown): Promise<void | string> {
    await pageData.refetch();
    const latestItems = (pageData.data?.categories ?? []).map((category) => ({
      ...category,
      id: category.id
    }));
    const recovery = recoverReorderConflict({
      controller: reorderController,
      error,
      latestItems,
      entityLabel: "category"
    });

    if (!recovery.handled) return;

    toastStore.push({
      variant: "info",
      message: recovery.message
    });
    return recovery.message;
  }

  function exitReorderMode() {
    reorderController.reset();
    reorderSubmitError = null;
    isReorderMode = false;
  }

  function handleReorderItems(items: typeof reorderController.pending) {
    reorderController.updatePending(items);
  }

  async function handleReorderSubmit() {
    reorderSubmitError = null;
    try {
      await reorderController.submit();
      await handleReorderSuccess();
    } catch (error) {
      const transformed = await handleReorderError(error);
      reorderSubmitError = transformed ?? (error instanceof Error ? error.message : String(error));
    }
  }

  async function handleDeleteCategory(categoryId: string) {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await adminCommands.softDeleteCategory(categoryId, fetch, token);
      toastStore.push({ variant: "success", message: "Category deleted" });
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete category";
      toastStore.push({ variant: "error", message });
    }
  }
</script>

<PoodlePageHeader title="Categories" backHref="/" backLabel="Back to dashboard">
  {#snippet actions()}
    {#if (pageData.data?.categories ?? []).length > 1}
      <PoodleIconButton
        type="button"
        variant="secondary"
        tone={isReorderMode ? "danger" : "default"}
        icon={arrowUpDownIcon}
        ariaLabel={isReorderMode ? "Cancel reorder" : "Reorder categories"}
        tooltip={isReorderMode ? "Cancel Reorder" : "Reorder Categories"}
        on:click={() => isReorderMode ? exitReorderMode() : enterReorderMode()}
      />
    {/if}
    <PoodleIconButton
      type="button"
      variant="primary"
      icon="plus"
      ariaLabel="Add category"
      tooltip="Add Category"
      on:click={() =>
        void gotoWithContext("/categories/new", {
          label: "Categories",
          href: "/categories",
          type: "list"
        })}
    />
  {/snippet}
</PoodlePageHeader>

{#if !isReorderMode}
  <FilterToolbar ariaLabel="Category filters" summaryText="Filters">
    <svelte:fragment slot="actions">
      <PoodleButton type="button" variant="ghost" size="sm" onclick={() => pageData.refetch()}>
        Refresh
      </PoodleButton>
    </svelte:fragment>
    <PoodleField id="categories-filter-name" label="Name" let:describedBy>
      <PoodleSearchField type="search"
        id="categories-filter-name"
        value={nameFilterInput}
        describedBy={describedBy}
        placeholder="Search by name..."
        on:valueChange={(event) => handleNameInput(event.detail.value)}
      />
    </PoodleField>
    <PoodleField id="categories-filter-status" label="Status" let:describedBy>
      <PoodleSelect
        id="categories-filter-status"
        value={selectedIsActive}
        describedBy={describedBy}
        options={activeItems}
        placeholder="All"
        on:valueChange={(event) => handleActiveChange(event.detail.value)}
      />
    </PoodleField>
    <PoodleField id="categories-filter-sort" label="Sort">
      <PoodleOrderBy fields={sortFields} value={orderBy} onChange={handleSortChange} compact />
    </PoodleField>
  </FilterToolbar>
{/if}

{#if pageData.loading}
  <PageLoading presentation="inline" message="Loading categories..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if (pageData.data?.categories ?? []).length === 0}
  <PoodleEmptyState title="No categories found" message="Create your first category to get started.">
    <a slot="actions" href="/categories/new">Add category</a>
  </PoodleEmptyState>
{:else if isReorderMode}
  <PoodleReorderableList
    items={reorderController.pending}
    ariaLabel="Reorder categories"
    dirty={reorderController.isDirty}
    submitting={reorderController.isPending}
    errorMessage={reorderSubmitError}
    onsubmit={handleReorderSubmit}
    oncancel={exitReorderMode}
    on:reorder={(event) => handleReorderItems(event.detail.items)}
  >
    {#snippet item(category)}
      <PoodleListCard
        title={category.name}
        layout="compact"
        showReorderHandle
        accentColor={category.color ?? "#6366f1"}
      >
        <svelte:fragment slot="leading">
          <FolderOpen size={16} />
        </svelte:fragment>
      </PoodleListCard>
    {/snippet}
  </PoodleReorderableList>
{:else}
  <PoodleGrid columns="repeat(auto-fit, minmax(min(26em, 100%), 1fr))" gap="lg">
    {#each pageData.data?.categories ?? [] as category}
      <CategoryListCard {category} onDelete={handleDeleteCategory} />
    {/each}
  </PoodleGrid>
{/if}
