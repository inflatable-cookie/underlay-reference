<script lang="ts">
  import { createPageListQueryState } from "@decodelabs/underlay/patterns";
  import { AuditLogList } from "$lib/lists";

  const listQuery = createPageListQueryState({ mode: "url" });
</script>

<AuditLogList query={listQuery.query} onQueryChange={listQuery.setQuery} />
