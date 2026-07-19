<script lang="ts">
  import { goto } from "$app/navigation";
  import { EntityListPage, toPagedListResult } from "@decodelabs/underlay/templates";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import type { QueryParams } from "@decodelabs/underlay/client/query";
  import { ProjectListCard } from "$lib/cards";
  import { adminCommands, type ProjectWithCounts } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";

  interface Props {
    title?: string;
    hideTitle?: boolean;
    subtitle?: string;
    eyebrow?: string;
    headerLevel?: 1 | 2 | 3 | 4 | 5 | 6;
    backHref?: string;
    backLabel?: string;
    categoryId?: string;
    query?: QueryParams;
    onQueryChange?: (query: QueryParams) => void;
  }

  let {
    title = "Projects",
    hideTitle = false,
    subtitle,
    eyebrow,
    headerLevel = 2,
    backHref,
    backLabel,
    categoryId,
    query,
    onQueryChange
  }: Props = $props();

  const toastStore = useToasts();
  let localQuery = $state<QueryParams>({ page: 1, limit: 30 });

  const effectiveQuery = $derived(query ?? localQuery);
  const hasFixedCategory = $derived(Boolean(categoryId));

  function withFixedFilters(input: QueryParams): QueryParams {
    const nextFilters = (input.filters ?? []).filter((filter) =>
      hasFixedCategory ? filter.field !== "categoryId" : true
    );

    if (categoryId) {
      nextFilters.push({ field: "categoryId", value: categoryId });
    }

    return {
      ...input,
      filters: nextFilters
    };
  }

  function updateQuery(nextQuery: QueryParams) {
    if (onQueryChange) {
      onQueryChange(nextQuery);
      return;
    }

    localQuery = nextQuery;
  }

  async function dataLoader(fetch: typeof window.fetch, token: string | null, nextQuery: QueryParams) {
    if (!token) throw new Error("Not authenticated");
    const response = await adminCommands.listProjects(fetch, token, withFixedFilters(nextQuery));
    return toPagedListResult(response);
  }

  async function handleBatchDelete(ids: string[]) {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    const result = await adminCommands.batchDeleteProjects({ ids }, fetch, token);
    toastStore.push({
      variant: "success",
      message: `Deleted ${result.deleted} project${result.deleted === 1 ? "" : "s"}`
    });
  }

  function handleAdd() {
    void gotoWithContext("/projects/new", {
      label: title,
      href: backHref ?? "/projects",
      type: "list"
    });
  }

  async function loadCategories(context?: { query?: string; value?: string | null }) {
    const token = auth.getToken();
    if (!token) return [{ value: "All", label: "All categories" }];
    const categories = await adminCommands.listCategoriesForSuggestions(fetch, token, {
      query: context?.query,
      limit: 20
    });
    if (context?.value && context.value !== "All" && !categories.some((category) => category.id === context.value)) {
      try {
        const category = await adminCommands.getCategory(context.value, fetch, token);
        categories.unshift(category);
      } catch {
        // Ignore stale selections and keep the rest of the option list.
      }
    }
    return [
      { value: "All", label: "All categories" },
      ...categories.map((c) => ({ value: c.id, label: c.name }))
    ];
  }

  async function handleDeleteProject(projectId: string) {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    await adminCommands.softDeleteProject(projectId, fetch, token);
    toastStore.push({ variant: "success", message: "Project deleted" });
  }

  const filters = $derived.by(() => [
    { id: "name", type: "search" as const, label: "Name", placeholder: "Search by name..." },
    ...(hasFixedCategory
      ? []
      : [{
          id: "categoryId",
          type: "select" as const,
          label: "Category",
          loadOptions: loadCategories,
          searchable: true
        }]),
    {
      id: "status",
      type: "select" as const,
      label: "Status",
      options: [
        { value: "All", label: "All statuses" },
        { value: "active", label: "Active" },
        { value: "archived", label: "Archived" },
        { value: "on_hold", label: "On Hold" }
      ]
    },
    {
      id: "sort",
      type: "sort" as const,
      label: "Sort",
      sortFields: [
        { key: "name", label: "Name" },
        { key: "weight", label: "Weight" },
        { key: "categoryName", label: "Category" },
        { key: "taskCount", label: "Tasks" },
        { key: "createdAt", label: "Created", defaultDirection: "desc" as const }
      ]
    }
  ]);

  const batchActions = [
    {
      id: "delete",
      label: "Delete",
      tone: "danger" as const,
      icon: "trash-2",
      confirm: {
        title: "Delete projects",
        description: (count: number) =>
          `Are you sure you want to delete ${count} project${count === 1 ? "" : "s"}?`,
        confirmLabel: "Delete projects",
        cancelLabel: "Keep projects"
      },
      handler: handleBatchDelete
    }
  ];

  const isProjectCollectionFiltered = $derived(
    hasFixedCategory ||
      (effectiveQuery.filters?.length ?? 0) > 0 ||
      (effectiveQuery.sort ?? []).some((field) => field.field !== "weight")
  );

  const reorderConfig = $derived(
    isProjectCollectionFiltered
      ? undefined
      : {
          enabled: true,
          handler: async (orderedIds: string[]) => {
            const token = auth.getToken();
            if (!token) throw new Error("Not authenticated");
            await adminCommands.reorderProjects({ ids: orderedIds }, fetch, token);
          }
        }
  );

  async function handleReorderError(error: unknown) {
    const message = error instanceof Error ? error.message : String(error);
    toastStore.push({ variant: "error", message });
    return message;
  }
</script>

{#snippet projectCard(project: ProjectWithCounts, ctx: { selectionMode: boolean; reorderMode: boolean; selected: boolean; onToggle: (selected: boolean) => void; refetch: () => Promise<void> })}
  <ProjectListCard
    {project}
    reorderMode={ctx.reorderMode}
    selectionMode={ctx.selectionMode}
    selected={ctx.selected}
    onSelectionChange={(_id, selected) => ctx.onToggle(selected)}
    onDelete={ctx.selectionMode ? undefined : async (id) => {
      await handleDeleteProject(id);
      await ctx.refetch();
    }}
  />
{/snippet}

<EntityListPage
  {title}
  {hideTitle}
  {subtitle}
  {eyebrow}
  {headerLevel}
  {backHref}
  {backLabel}
  {dataLoader}
  presentation="cards"
  renderItem={projectCard}
  filters={filters}
  query={effectiveQuery}
  {batchActions}
  onQueryChange={updateQuery}
  reorder={reorderConfig}
  onReorderError={handleReorderError}
  onAdd={handleAdd}
  addLabel="Add Project"
/>
