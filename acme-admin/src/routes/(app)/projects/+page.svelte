<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { EntityListPage } from "@decodelabs/underlay/templates";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import { buildQueryString, parseQueryParams } from "@decodelabs/underlay/client/query";
  import type { QueryParams } from "@decodelabs/underlay/client/query";
  import { ProjectListCard } from "$lib/cards";
  import { adminCommands, type ProjectWithCounts } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";

  const toastStore = useToasts();

  // Parse current state from URL
  const currentQuery = $derived(parseQueryParams($page.url.searchParams));

  function updateUrl(nextQuery: QueryParams) {
    const url = new URL($page.url);
    const queryString = buildQueryString(nextQuery);
    url.search = queryString;
    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  async function dataLoader(fetch: typeof window.fetch, token: string | null, query: QueryParams) {
    if (!token) throw new Error("Not authenticated");
    return await adminCommands.listProjects(fetch, token, query);
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
      label: "Projects",
      href: "/projects",
      type: "list"
    });
  }

  async function loadCategories() {
    const token = auth.getToken();
    if (!token) return [{ value: "All", label: "All categories" }];
    const categories = await adminCommands.listCategoriesForSuggestions(fetch, token);
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

  const filters = [
    { id: "name", type: "search" as const, label: "Name", placeholder: "Search by name..." },
    { id: "categoryId", type: "select" as const, label: "Category", loadOptions: loadCategories },
    { id: "status", type: "select" as const, label: "Status", options: [
      { value: "All", label: "All statuses" },
      { value: "active", label: "Active" },
      { value: "archived", label: "Archived" },
      { value: "on_hold", label: "On Hold" }
    ]},
    { id: "sort", type: "sort" as const, label: "Sort", sortFields: [
      { key: "name", label: "Name" },
      { key: "weight", label: "Weight" },
      { key: "categoryName", label: "Category" },
      { key: "taskCount", label: "Tasks" },
      { key: "createdAt", label: "Created", defaultDirection: "desc" as const }
    ]}
  ];

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
    (currentQuery.filters?.length ?? 0) > 0 ||
      (currentQuery.sort ?? []).some((field) => field.field !== "weight")
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
</script>

{#snippet projectCard(project: ProjectWithCounts, ctx: { selectionMode: boolean; reorderMode: boolean; selected: boolean; onToggle: (selected: boolean) => void; refetch: () => Promise<void> })}
  <ProjectListCard
    {project}
    reorderMode={ctx.reorderMode}
    selectionMode={ctx.selectionMode}
    selected={ctx.selected}
    onSelectionChange={(id, selected) => ctx.onToggle(selected)}
    onDelete={ctx.selectionMode ? undefined : async (id) => {
      await handleDeleteProject(id);
      await ctx.refetch();
    }}
  />
{/snippet}

<EntityListPage
  title="Projects"
  backHref="/"
  backLabel="Back to dashboard"
  {dataLoader}
  presentation="cards"
  renderItem={projectCard as never}
  {filters}
  query={currentQuery}
  {batchActions}
  onQueryChange={updateUrl}
  reorder={reorderConfig as never}
  onAdd={handleAdd}
  addLabel="Add Project"
/>
