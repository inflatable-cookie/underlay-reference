<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { buildQueryString, parseQueryParams, type QueryParams } from "@decodelabs/underlay/client/query";
  import { SystemMediaTrashListPage } from "@decodelabs/underlay/templates";
  import { mediaCommands } from "@api-client";
  import type { SystemMediaTrashItem } from "@decodelabs/underlay/templates";

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
  const currentQuery = $derived(parseQueryParams($page.url.searchParams));

  function updateUrl(nextQuery: QueryParams) {
    const url = new URL($page.url);
    url.search = buildQueryString(nextQuery);
    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

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
  query={currentQuery}
  onQueryChange={updateUrl}
  {restoreAction}
  {purgeAction}
/>
