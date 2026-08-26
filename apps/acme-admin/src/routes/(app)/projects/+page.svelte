<script lang="ts">
  import { createPageListQueryState } from "@inflatable-cookie/underlay/patterns";
  import ProjectsListPage from "$lib/lists/ProjectsListPage.svelte";

  const listQuery = createPageListQueryState({ mode: "url" });
</script>

<ProjectsListPage
  title="Projects"
  backHref="/"
  backLabel="Back to dashboard"
  query={listQuery.query}
  onQueryChange={listQuery.setQuery}
/>
