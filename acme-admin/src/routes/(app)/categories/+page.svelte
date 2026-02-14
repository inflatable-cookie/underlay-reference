<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import {
    FilterBar,
    PageHeader,
    ReorderableList,
    createReorderController,
    useToasts,
    useAuthenticatedData
  } from "@decodelabs/underlay/patterns";
  import {
    Button,
    EmptyState,
    Field,
    FormError,
    ListGrid,
    ListCard,
    OrderBy,
    PageLoading,
    Select,
    TextInput,
    Tooltip,
    type OrderByValue
  } from "@decodelabs/underlay/components";
  import { gotoWithContext, parseQueryParams } from "@decodelabs/underlay/client";
  import { CategoryListCard } from "$lib/cards";
  import { adminCommands, type CategoryWithCounts } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
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
      getToken: () => auth.getToken(),
      defaultValue: { categories: [] as CategoryWithCounts[] },
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

<PageHeader section="Categories" backHref="/" backLabel="Back to dashboard">
  {#snippet actions()}
    {#if (pageData.data?.categories ?? []).length > 1}
      <Tooltip content={isReorderMode ? "Cancel Reorder" : "Reorder Categories"} inline>
        {#snippet trigger()}
          <Button
            type="button"
            variant={isReorderMode ? "danger" : "subtle"}
            size="icon"
            onclick={() => isReorderMode ? exitReorderMode() : enterReorderMode()}
          >
            <ArrowUpDown size={16} />
          </Button>
        {/snippet}
      </Tooltip>
    {/if}
    <Tooltip content="Add Category" inline>
      {#snippet trigger()}
        <Button
          type="button"
          variant="primary"
          size="icon"
          onclick={() =>
            void gotoWithContext("/categories/new", {
              label: "Categories",
              href: "/categories",
              type: "list"
            })}
        >
          <Plus size={16} />
        </Button>
      {/snippet}
    </Tooltip>
  {/snippet}
</PageHeader>

{#if !isReorderMode}
  <FilterBar title="Filters" onRefresh={() => pageData.refetch()}>
    <Field label="Name" forId="name">
      <TextInput
        id="name"
        value={selectedName.replace(/%/g, "")}
        onchange={handleNameChange}
        debounce={500}
        search
        placeholder="Filter by name..."
      />
    </Field>
    <Field label="Status" forId="isActive">
      <Select
        id="isActive"
        value={selectedIsActive}
        onchange={handleActiveChange}
        items={activeItems}
        placeholder="All"
        clearable
        defaultValue="All"
      />
    </Field>
    <Field label="Sort" forId="sort">
      <OrderBy fields={sortFields} value={orderBy} onChange={handleSortChange} />
    </Field>
  </FilterBar>
{/if}

{#if pageData.loading}
  <PageLoading message="Loading categories..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if (pageData.data?.categories ?? []).length === 0}
  <EmptyState title="No categories found" description="Create your first category to get started." actionLabel="Add category" actionHref="/categories/new" />
{:else if isReorderMode}
  <ReorderableList controller={reorderController} oncancel={exitReorderMode} onsuccess={handleReorderSuccess}>
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

