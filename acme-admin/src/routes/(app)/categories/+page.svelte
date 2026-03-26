<script lang="ts">
  import { PageHeader as PoodlePageHeader } from "@poodle/svelte-composites";
  import { Callout as PoodleCallout } from "@poodle/svelte-primitives";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import {
    FilterBar,
    ReorderableList,
    createReorderController,
    useToasts,
    useAuthenticatedData
  } from "@decodelabs/underlay/patterns";
  import {
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
    SearchField as PoodleSearchField,
    Select as PoodleSelect
  } from "@poodle/svelte-primitives";
  import { gotoWithContext, parseQueryParams } from "@decodelabs/underlay/client";
  import { CategoryListCard } from "$lib/cards";
  import { recoverReorderConflict } from "$lib/lists/reorder-conflicts";
  import { arrowUpDownIcon } from "$lib/ui/poodle-icon-nodes";
  import { adminCommands, type CategoryWithCounts } from "acme-client";
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
    isReorderMode = true;
  }

  async function handleReorderSuccess() {
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
    isReorderMode = false;
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
  <svelte:fragment slot="actions">
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
  </svelte:fragment>
</PoodlePageHeader>

{#if !isReorderMode}
  <FilterBar title="Filters" onRefresh={() => pageData.refetch()}>
    <PoodleField id="categories-filter-name" label="Name" let:describedBy>
      <PoodleSearchField
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
      <OrderBy fields={sortFields} value={orderBy} onChange={handleSortChange} />
    </PoodleField>
  </FilterBar>
{/if}

{#if pageData.loading}
  <PageLoading message="Loading categories..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if (pageData.data?.categories ?? []).length === 0}
  <EmptyState title="No categories found" description="Create your first category to get started." actionLabel="Add category" actionHref="/categories/new" />
{:else if isReorderMode}
  <ReorderableList
    controller={reorderController}
    oncancel={exitReorderMode}
    onsuccess={handleReorderSuccess}
    onsubmiterror={handleReorderError}
  >
    {#snippet item(category)}
      <ListCard
        title={category.name}
        variant="compact"
        showDragHandle
        accent={category.color ?? "#6366f1"}
      >
        {#snippet media()}
          <FolderOpen size={16} />
        {/snippet}
      </ListCard>
    {/snippet}
  </ReorderableList>
{:else}
  <ListGrid minItemWidth={26}>
    {#each pageData.data?.categories ?? [] as category}
      <CategoryListCard {category} onDelete={handleDeleteCategory} />
    {/each}
  </ListGrid>
{/if}
