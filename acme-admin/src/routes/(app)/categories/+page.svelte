<script lang="ts">
  import { createPageListQueryState } from "@decodelabs/underlay/patterns";
  import { CategoriesList } from "$lib/lists";

  const listQuery = createPageListQueryState({ mode: "url" });
</script>

<CategoriesList query={listQuery.query} onQueryChange={listQuery.setQuery} />
