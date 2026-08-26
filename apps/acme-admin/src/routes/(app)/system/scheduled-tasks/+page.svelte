<script lang="ts">
  import { createPageListQueryState } from "@inflatable-cookie/underlay/patterns";
  import { ScheduledTasksList } from "$lib/lists";

  const listQuery = createPageListQueryState({ mode: "url" });
</script>

<ScheduledTasksList query={listQuery.query} onQueryChange={listQuery.setQuery} />
