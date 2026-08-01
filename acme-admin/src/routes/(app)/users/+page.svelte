<script lang="ts">
  import { createPageListQueryState } from "@decodelabs/underlay/patterns";
  import { UsersList } from "$lib/lists";

  const listQuery = createPageListQueryState({ mode: "url" });
</script>

<UsersList query={listQuery.query} onQueryChange={listQuery.setQuery} />
