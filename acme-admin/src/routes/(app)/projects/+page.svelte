<script lang="ts">
  import { PageHeader as PoodlePageHeader } from "@poodle/svelte-composites";
  import { Callout as PoodleCallout } from "@poodle/svelte-primitives";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import {
    FilterBar,
    ReorderableList,
    createReorderController,
    useToasts,
    useAuthenticatedData,
    useBatchSelection
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
    SearchField as PoodleSearchField,
    Select as PoodleSelect
  } from "@poodle/svelte-primitives";
  import { gotoWithContext, parseQueryParams } from "@decodelabs/underlay/client";
  import { ProjectListCard } from "$lib/cards";
  import { recoverReorderConflict } from "$lib/lists/reorder-conflicts";
  import { arrowUpDownIcon, squareCheckIcon } from "$lib/ui/poodle-icon-nodes";
  import { adminCommands, type ProjectWithCounts } from "acme-client";
  import { auth } from "$lib/stores/auth";
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
      defaultValue: { projects: [] as ProjectWithCounts[] },
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

  const defaultCategoryOptions = [{ value: "All", label: "All categories" }];
  let categoryOptions = $state(defaultCategoryOptions);
  let nameFilterInput = $state("");
  let nameFilterTimer: ReturnType<typeof setTimeout> | null = null;

  onMount(async () => {
    const token = auth.getToken();
    if (!token) {
      categoryOptions = defaultCategoryOptions;
      return;
    }

    try {
      const categories = await adminCommands.listCategoriesForSuggestions(fetch, token);
      categoryOptions = [
        ...defaultCategoryOptions,
        ...categories.map((category) => ({ value: category.id, label: category.name }))
      ];
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to load categories";
      toastStore.push({ variant: "error", message });
      categoryOptions = defaultCategoryOptions;
    }
  });

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

  async function handleReorderError(error: unknown): Promise<void | string> {
    await pageData.refetch();
    const latestItems = (pageData.data?.projects ?? []).map((project) => ({ ...project, id: project.id }));
    const recovery = recoverReorderConflict({
      controller: reorderController,
      error,
      latestItems,
      entityLabel: "project"
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

<PoodlePageHeader title="Projects" backHref="/" backLabel="Back to dashboard">
  <svelte:fragment slot="actions">
    {#if (pageData.data?.projects ?? []).length > 0 && !isReorderMode}
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
    {#if (pageData.data?.projects ?? []).length > 1 && !isSelectionMode}
      <PoodleIconButton
        type="button"
        variant="secondary"
        tone={isReorderMode ? "danger" : "default"}
        icon={arrowUpDownIcon}
        ariaLabel={isReorderMode ? "Cancel reorder" : "Reorder projects"}
        tooltip={isReorderMode ? "Cancel Reorder" : "Reorder Projects"}
        on:click={() => isReorderMode ? exitReorderMode() : enterReorderMode()}
      />
    {/if}
    {#if !isSelectionMode && !isReorderMode}
      <PoodleIconButton
        type="button"
        variant="primary"
        icon="plus"
        ariaLabel="Add project"
        tooltip="Add Project"
        on:click={() =>
          void gotoWithContext("/projects/new", {
            label: "Projects",
            href: "/projects",
            type: "list"
          })}
      />
    {/if}
  </svelte:fragment>
</PoodlePageHeader>

{#if !isReorderMode}
  <FilterBar title="Filters" onRefresh={() => pageData.refetch()}>
    <PoodleField id="projects-filter-name" label="Name" let:describedBy>
      <PoodleSearchField
        id="projects-filter-name"
        value={nameFilterInput}
        describedBy={describedBy}
        placeholder="Search by name..."
        on:valueChange={(event) => handleNameInput(event.detail.value)}
      />
    </PoodleField>
    <PoodleField id="projects-filter-category" label="Category" let:describedBy>
      <PoodleSelect
        id="projects-filter-category"
        value={selectedCategoryId}
        describedBy={describedBy}
        options={categoryOptions}
        placeholder="All categories"
        on:valueChange={(event) => handleCategoryChange(event.detail.value)}
      />
    </PoodleField>
    <PoodleField id="projects-filter-status" label="Status" let:describedBy>
      <PoodleSelect
        id="projects-filter-status"
        value={selectedStatus}
        describedBy={describedBy}
        options={statusItems}
        placeholder="All statuses"
        on:valueChange={(event) => handleStatusChange(event.detail.value)}
      />
    </PoodleField>
    <PoodleField id="projects-filter-sort" label="Sort">
      <OrderBy fields={sortFields} value={orderBy} onChange={handleSortChange} />
    </PoodleField>
  </FilterBar>
{/if}

{#if pageData.loading}
  <PageLoading message="Loading projects..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if (pageData.data?.projects ?? []).length === 0}
  <EmptyState title="No projects found" description="Create your first project to get started." actionLabel="Add project" actionHref="/projects/new" />
{:else if isReorderMode}
  <ReorderableList
    controller={reorderController}
    oncancel={exitReorderMode}
    onsuccess={handleReorderSuccess}
    onsubmiterror={handleReorderError}
  >
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
