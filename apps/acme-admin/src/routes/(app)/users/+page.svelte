<script lang="ts">
  import { createPageListQueryState } from "@inflatable-cookie/underlay/patterns";
  import { UsersList } from "$lib/lists";

  const listQuery = createPageListQueryState({ mode: "url" });
</script>

<UsersList query={listQuery.query} onQueryChange={listQuery.setQuery} />
