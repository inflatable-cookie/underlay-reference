<script lang="ts">
  import { createEntityListState } from "@inflatable-cookie/underlay/patterns";
  import type { QueryParams } from "@inflatable-cookie/underlay/client/query";
  import { SystemMediaTrashListPage } from "@inflatable-cookie/underlay/templates";
  import { mediaCommands } from "@api-client";
  import type { SystemMediaTrashItem } from "@inflatable-cookie/underlay/templates";

  interface Props {
    title?: string;
    backHref?: string;
    backLabel?: string;
  }

  let {
    title = "Media Trash",
    backHref = "/media",
    backLabel = "Back to media"
  }: Props = $props();

  const listState = createEntityListState({
    queryMode: () => "url",
    title: () => title
  });

  async function dataLoader(fetch: typeof globalThis.fetch, token: string, query: QueryParams) {
    return await mediaCommands.listMediaTrash(fetch, token, query);
  }

  async function restoreAction(media: SystemMediaTrashItem, fetch: typeof globalThis.fetch, token: string) {
    await mediaCommands.restoreMedia(media.id, fetch, token);
  }

  async function purgeAction(media: SystemMediaTrashItem, fetch: typeof globalThis.fetch, token: string) {
    await mediaCommands.purgeMedia(media.id, fetch, token);
  }
</script>

<SystemMediaTrashListPage
  {title}
  {backHref}
  {backLabel}
  {dataLoader}
  query={listState.query}
  onQueryChange={listState.setQuery}
  {restoreAction}
  {purgeAction}
/>
