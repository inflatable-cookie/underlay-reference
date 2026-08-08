<script lang="ts">
  import { createPageListQueryState } from "@inflatable-cookie/underlay/patterns";
  import { CategoriesList } from "$lib/lists";

  const listQuery = createPageListQueryState({ mode: "url" });
</script>

<CategoriesList query={listQuery.query} onQueryChange={listQuery.setQuery} />
