<script lang="ts">
  import { gotoWithContext } from "@inflatable-cookie/underlay/client/navigation";
  import { createEntityListState } from "@inflatable-cookie/underlay/patterns";
  import type { QueryParams } from "@inflatable-cookie/underlay/client/query";
  import type { NavigationContext } from "@inflatable-cookie/underlay/runtime/navigation";
  import { EntityListPage, toPagedListResult, type FilterConfig } from "@inflatable-cookie/underlay/templates";
  import { adminCommands, type Label } from "@api-client";
  import { LabelListCard } from "$lib/cards";

  interface Props {
    projectId: string;
    title?: string;
    hideTitle?: boolean;
    subtitle?: string;
    eyebrow?: string;
    headerLevel?: 1 | 2 | 3 | 4 | 5 | 6;
    backHref?: string;
    backLabel?: string;
    queryMode?: "url" | "local";
    sourceContext?: NavigationContext;
    onDataChange?: () => void;
  }

  let {
    projectId,
    title = "Labels",
    hideTitle = false,
    subtitle,
    eyebrow,
    headerLevel = 2,
    backHref = undefined,
    backLabel = "Back to project",
    queryMode = "url",
    sourceContext: providedSourceContext = undefined,
    onDataChange
  }: Props = $props();

  const listState = createEntityListState({
    queryMode: () => queryMode,
    title: () => title,
    sourceContext: () => providedSourceContext,
    reloadScope: () => `labels:${projectId}`
  });
  const currentQuery = $derived(listState.query);
  const reloadKey = $derived(listState.reloadKey);
  const sourceContext = $derived(listState.sourceContext);

  const filters: FilterConfig[] = [
    {
      id: "name",
      type: "search",
      label: "Name",
      placeholder: "Search labels..."
    }
  ];

  async function dataLoader(fetchFn: typeof fetch, token: string | null, query: QueryParams) {
    if (!token) throw new Error("Not authenticated");
    const response = await adminCommands.listLabels(projectId, fetchFn, token, query);
    return toPagedListResult(response);
  }

  function handleAdd() {
    void gotoWithContext(`/projects/${projectId}/labels/new`, sourceContext);
  }
</script>

{#snippet labelCard(label: Label)}
  <LabelListCard {label} {sourceContext} />
{/snippet}

<EntityListPage
  {title}
  {hideTitle}
  {subtitle}
  {eyebrow}
  {headerLevel}
  {...listState.backHrefProps(backHref ?? `/projects/${projectId}`, backLabel)}
  {dataLoader}
  {reloadKey}
  presentation="cards"
  renderItem={labelCard}
  {filters}
  query={currentQuery}
  onQueryChange={listState.setQuery}
  {onDataChange}
  onAdd={handleAdd}
  addLabel="Add label"
/>
