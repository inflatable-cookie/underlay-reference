<script lang="ts">
  import { createPageListQueryState } from "@decodelabs/underlay/patterns";
  import { JobsList } from "$lib/lists";

  const listQuery = createPageListQueryState({ mode: "url" });
</script>

<JobsList query={listQuery.query} onQueryChange={listQuery.setQuery} />
