<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { buildQueryString, parseQueryParams } from "@decodelabs/underlay/client/query";
  import type { QueryParams } from "@decodelabs/underlay/client/query";
  import ProjectsListPage from "$lib/lists/ProjectsListPage.svelte";

  const currentQuery = $derived(parseQueryParams($page.url.searchParams));

  function updateUrl(nextQuery: QueryParams) {
    const url = new URL($page.url);
    url.search = buildQueryString(nextQuery);
    goto(url.toString(), { replaceState: true, keepFocus: true });
  }
</script>

<ProjectsListPage
  title="Projects"
  backHref="/"
  backLabel="Back to dashboard"
  query={currentQuery}
  onQueryChange={updateUrl}
/>
