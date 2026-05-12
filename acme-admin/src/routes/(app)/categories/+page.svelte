<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { buildQueryString, parseQueryParams, type QueryParams } from "@decodelabs/underlay/client/query";
  import { CategoriesList } from "$lib/lists";

  const currentQuery = $derived(parseQueryParams($page.url.searchParams));

  function updateUrl(nextQuery: QueryParams) {
    const url = new URL($page.url);
    url.search = buildQueryString(nextQuery);
    goto(url.toString(), { replaceState: true, keepFocus: true });
  }
</script>

<CategoriesList query={currentQuery} onQueryChange={updateUrl} />
