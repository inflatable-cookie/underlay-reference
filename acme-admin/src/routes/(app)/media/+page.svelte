<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { buildQueryString, parseQueryParams, type QueryParams } from "@inflatable-cookie/underlay/client/query";
  import { MediaList } from "$lib/lists";

  const currentQuery = $derived(parseQueryParams($page.url.searchParams));

  function updateUrl(nextQuery: QueryParams) {
    const url = new URL($page.url);
    url.search = buildQueryString(nextQuery);
    goto(url.toString(), { replaceState: true, keepFocus: true });
  }
</script>

<MediaList query={currentQuery} onQueryChange={updateUrl} />
