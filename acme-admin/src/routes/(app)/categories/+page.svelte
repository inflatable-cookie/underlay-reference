<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { EntityListPage, toPagedListResult } from "@decodelabs/underlay/templates";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import { buildQueryString, parseQueryParams } from "@decodelabs/underlay/client/query";
  import type { QueryParams } from "@decodelabs/underlay/client/query";
  import { CategoryListCard } from "$lib/cards";
  import { adminCommands, type CategoryWithCounts } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";

  const toastStore = useToasts();

  const currentQuery = $derived(parseQueryParams($page.url.searchParams));

  function updateUrl(nextQuery: QueryParams) {
    const url = new URL($page.url);
    url.search = buildQueryString(nextQuery);
    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  async function dataLoader(fetch: typeof window.fetch, token: string | null, query: QueryParams) {
    if (!token) throw new Error("Not authenticated");
    const response = await adminCommands.listCategories(fetch, token, query);
    return toPagedListResult(response);
  }

  async function handleDeleteCategory(categoryId: string) {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    await adminCommands.softDeleteCategory(categoryId, fetch, token);
    toastStore.push({ variant: "success", message: "Category deleted" });
  }

  async function handleBatchDelete(ids: string[]) {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    const result = await adminCommands.batchDeleteCategories({ ids }, fetch, token);
    toastStore.push({
      variant: "success",
      message: `Deleted ${result.deleted} categor${result.deleted === 1 ? "y" : "ies"}`
    });
  }

  function handleAdd() {
    void gotoWithContext("/categories/new", {
      label: "Categories",
      href: "/categories",
      type: "list"
    });
  }

  const filters = [
    {
      id: "name",
      type: "search" as const,
      label: "Name",
      placeholder: "Search by name..."
    },
    {
      id: "isActive",
      type: "select" as const,
      label: "Status",
      options: [
        { value: "All", label: "All statuses" },
        { value: "true", label: "Active" },
        { value: "false", label: "Inactive" }
      ]
    },
    {
      id: "sort",
      type: "sort" as const,
      label: "Sort",
      sortFields: [
        { key: "name", label: "Name" },
        { key: "weight", label: "Weight" },
        { key: "projectCount", label: "Projects" },
        { key: "createdAt", label: "Created", defaultDirection: "desc" as const }
      ]
    }
  ];

  const canUseCanonicalCategoryOrder = $derived(
    (currentQuery.filters?.length ?? 0) === 0 &&
      (currentQuery.sort ?? []).every((field) => field.field === "weight")
  );

  const reorderConfig = $derived(
    canUseCanonicalCategoryOrder
      ? {
          enabled: true,
          handler: async (orderedIds: string[]) => {
            const token = auth.getToken();
            if (!token) throw new Error("Not authenticated");
            await adminCommands.reorderCategories({ ids: orderedIds }, fetch, token);
          }
        }
      : undefined
  );

  const batchActions = [
    {
      id: "delete",
      label: "Delete",
      tone: "danger" as const,
      icon: "trash-2",
      confirm: {
        title: "Delete categories",
        description: (count: number) =>
          `Are you sure you want to delete ${count} categor${count === 1 ? "y" : "ies"}?`,
        confirmLabel: "Delete categories",
        cancelLabel: "Keep categories"
      },
      handler: handleBatchDelete
    }
  ];

  async function handleReorderError(error: unknown) {
    const message = error instanceof Error ? error.message : String(error);
    toastStore.push({
      variant: "error",
      message
    });
    return message;
  }
</script>

{#snippet categoryCard(category: CategoryWithCounts, ctx: { selectionMode: boolean; reorderMode: boolean; selected: boolean; onToggle: (selected: boolean) => void; refetch: () => Promise<void> })}
  <CategoryListCard
    {category}
    reorderMode={ctx.reorderMode}
    selectionMode={ctx.selectionMode}
    selected={ctx.selected}
    onSelectionChange={(categoryId, nextSelected) => {
      if (categoryId !== category.id) return;
      ctx.onToggle(nextSelected);
    }}
    onDelete={ctx.selectionMode || ctx.reorderMode ? undefined : async (id) => {
      await handleDeleteCategory(id);
      await ctx.refetch();
    }}
  />
{/snippet}

<EntityListPage
  title="Categories"
  backHref="/"
  backLabel="Back to dashboard"
  {dataLoader}
  presentation="cards"
  renderItem={categoryCard as never}
  {filters}
  query={currentQuery}
  {batchActions}
  onQueryChange={updateUrl}
  reorder={reorderConfig as never}
  onAdd={handleAdd}
  addLabel="Add category"
  onReorderError={handleReorderError}
/>
