<script lang="ts">
  import { LabelsList } from "$lib/lists";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();
</script>

<LabelsList projectId={data.projectId} />
