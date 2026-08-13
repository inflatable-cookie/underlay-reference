<script lang="ts">
  import { createPageListQueryState } from "@inflatable-cookie/underlay/patterns";
  import { MediaList } from "$lib/lists";

  const listQuery = createPageListQueryState({ mode: "url" });
</script>

<MediaList query={listQuery.query} onQueryChange={listQuery.setQuery} />
