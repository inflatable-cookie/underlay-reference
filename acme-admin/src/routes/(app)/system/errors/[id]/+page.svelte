<script lang="ts">
  import { page } from "$app/stores";
  import { adminCommands } from "@api-client";
  import { ErrorLogDetailPage } from "@decodelabs/underlay/templates";

  const errorLogId = $derived.by(() => $page.params.id ?? "");

  async function dataLoader(id: string, fetch: typeof globalThis.fetch, token: string) {
    return await adminCommands.getErrorLog(id, fetch, token);
  }
</script>

<ErrorLogDetailPage
  id={errorLogId}
  {dataLoader}
/>
