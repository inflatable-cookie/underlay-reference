<script lang="ts">
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

  async function dataLoader(fetch: typeof globalThis.fetch, token: string) {
    return await mediaCommands.listMediaTrash(fetch, token);
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
  {restoreAction}
  {purgeAction}
/>
