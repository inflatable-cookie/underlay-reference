<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import {
    FilterBar,
    PageHeader,
    ReorderableList,
    createReorderController,
    useToasts,
    useAuthenticatedData,
    useBatchSelection
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
  import { ProjectListCard } from "$lib/cards";
  import { BatchActionBar } from "$lib/components";
  import { adminCommands, type ProjectWithCounts } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import ArrowUpDown from "lucide-svelte/icons/arrow-up-down";
  import Plus from "lucide-svelte/icons/plus";
  import Briefcase from "lucide-svelte/icons/briefcase";
  import CheckSquare from "lucide-svelte/icons/check-square";

  const toastStore = useToasts();

  // Track URL for refetching when filters change
  let previousUrl = $state<string | null>(null);

  // Fetch projects
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const query = parseQueryParams($page.url.searchParams);
      const projects = await adminCommands.listProjects(fetch, token, query);
      return { projects };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { projects: [] as ProjectWithCounts[] },
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
  let isSelectionMode = $state(false);
  const selection = useBatchSelection<string>();
  let batchLoading = $state(false);

  // Clear selection when exiting selection mode
  $effect(() => {
    if (!isSelectionMode) {
      selection.clear();
    }
  });

  // Selection handlers
  function toggleSelectionMode() {
    isSelectionMode = !isSelectionMode;
  }

  function handleClearSelection() {
    selection.clear();
    isSelectionMode = false;
  }

  function handleSelectAll() {
    const allIds = (pageData.data?.projects ?? []).map(p => p.id);
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
      const result = await adminCommands.batchDeleteProjects(
        { ids: selection.selectedIds },
        fetch,
        token
      );
      toastStore.push({
        variant: "success",
        message: `Deleted ${result.deleted} project${result.deleted === 1 ? "" : "s"}`
      });
      selection.clear();
      isSelectionMode = false;
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete projects";
      toastStore.push({ variant: "error", message });
    } finally {
      batchLoading = false;
    }
  }

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
  const selectedCategoryId = $derived(
    currentQuery.filters?.find((f) => f.field === "categoryId")?.value ?? "All"
  );
  const selectedStatus = $derived(
    currentQuery.filters?.find((f) => f.field === "status")?.value ?? "All"
  );

  const sortFields = [
    { key: "name", label: "Name" },
    { key: "weight", label: "Weight" },
    { key: "categoryName", label: "Category" },
    { key: "taskCount", label: "Tasks" },
    { key: "createdAt", label: "Created", defaultDirection: "desc" as const }
  ];

  const statusItems = [
    { value: "All", label: "All statuses" },
    { value: "active", label: "Active" },
    { value: "archived", label: "Archived" },
    { value: "on_hold", label: "On Hold" }
  ];

  async function loadCategoryItems() {
    const token = auth.getToken();
    if (!token) return [{ value: "All", label: "All categories" }];
    const categories = await adminCommands.listCategoriesForSuggestions(fetch, token);
    return [
      { value: "All", label: "All categories" },
      ...categories.map((c) => ({ value: c.id, label: c.name }))
    ];
  }

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

  // Update URL when category filter changes
  function handleCategoryChange(value: string) {
    const url = new URL($page.url);

    if (value && value !== "All") {
      url.searchParams.set("filter[categoryId]", value);
    } else {
      url.searchParams.delete("filter[categoryId]");
    }

    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  // Update URL when status filter changes
  function handleStatusChange(value: string) {
    const url = new URL($page.url);

    if (value && value !== "All") {
      url.searchParams.set("filter[status]", value);
    } else {
      url.searchParams.delete("filter[status]");
    }

    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  // Map projects to have 'id' field for reorder controller
  const reorderItems = $derived(
    (pageData.data?.projects ?? []).map((p) => ({ ...p, id: p.id }))
  );

  // Create reorder controller
  const reorderController = $derived(
    createReorderController(reorderItems, async (orderedIds) => {
      const token = auth.getToken();
      if (!token) {
        toastStore.push({ variant: "error", message: "Not authenticated" });
        return;
      }
      await adminCommands.reorderProjects({ ids: orderedIds }, fetch, token);
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

  async function handleDeleteProject(projectId: string) {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await adminCommands.softDeleteProject(projectId, fetch, token);
      toastStore.push({ variant: "success", message: "Project deleted" });
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete project";
      toastStore.push({ variant: "error", message });
    }
  }
</script>

<PageHeader section="Projects" backHref="/" backLabel="Back to dashboard">
  {#snippet actions()}
    {#if (pageData.data?.projects ?? []).length > 0 && !isReorderMode}
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
    {#if (pageData.data?.projects ?? []).length > 1 && !isSelectionMode}
      <Tooltip content={isReorderMode ? "Cancel Reorder" : "Reorder Projects"} inline>
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
    {#if !isSelectionMode && !isReorderMode}
      <Tooltip content="Add Project" inline>
        {#snippet trigger()}
          <Button
            type="button"
            variant="primary"
            size="icon"
            onclick={() =>
              void gotoWithContext("/projects/new", {
                label: "Projects",
                href: "/projects",
                type: "list"
              })}
          >
            <Plus size={16} />
          </Button>
        {/snippet}
      </Tooltip>
    {/if}
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
    <Field label="Category" forId="category">
      <Select
        id="category"
        value={selectedCategoryId}
        onchange={handleCategoryChange}
        loadItems={loadCategoryItems}
        placeholder="All categories"
        clearable
        defaultValue="All"
      />
    </Field>
    <Field label="Status" forId="status">
      <Select
        id="status"
        value={selectedStatus}
        onchange={handleStatusChange}
        items={statusItems}
        placeholder="All statuses"
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
  <PageLoading message="Loading projects..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if (pageData.data?.projects ?? []).length === 0}
  <EmptyState title="No projects found" description="Create your first project to get started." actionLabel="Add project" actionHref="/projects/new" />
{:else if isReorderMode}
  <ReorderableList controller={reorderController} oncancel={exitReorderMode} onsuccess={handleReorderSuccess}>
    {#snippet item(project)}
      <ListCard
        title={project.name}
        variant="compact"
        showDragHandle
      >
        {#snippet media()}
          <Briefcase size={16} />
        {/snippet}
      </ListCard>
    {/snippet}
  </ReorderableList>
{:else}
  <ListGrid minItemWidth={26}>
    {#each pageData.data?.projects ?? [] as project}
      <ProjectListCard
        {project}
        onDelete={isSelectionMode ? undefined : handleDeleteProject}
        selectionMode={isSelectionMode}
        selected={selection.isSelected(project.id)}
        onSelectionChange={(id, selected) => selection.toggle(id, selected)}
      />
    {/each}
  </ListGrid>
{/if}

<BatchActionBar
  selectedCount={selection.count}
  totalCount={(pageData.data?.projects ?? []).length}
  loading={batchLoading}
  onClearSelection={handleClearSelection}
  onSelectAll={handleSelectAll}
  onBatchDelete={handleBatchDelete}
/>

