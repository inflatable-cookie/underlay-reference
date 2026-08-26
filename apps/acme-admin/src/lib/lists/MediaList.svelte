<script lang="ts">
  import { goto } from "$app/navigation";
  import { gotoWithContext } from "@inflatable-cookie/underlay/client/navigation";
  import type { QueryParams } from "@inflatable-cookie/underlay/client/query";
  import { MediaListPage, toPagedListResult } from "@inflatable-cookie/underlay/templates";
  import { mediaCommands } from "@api-client";
  import { auth } from "$lib/stores/auth";

  interface Props {
    title?: string;
    hideTitle?: boolean;
    subtitle?: string;
    eyebrow?: string;
    headerLevel?: 1 | 2 | 3 | 4 | 5 | 6;
    backHref?: string;
    backLabel?: string;
    query: QueryParams;
    onQueryChange: (query: QueryParams) => void;
  }

  let {
    title = "Media Library",
    hideTitle = false,
    subtitle,
    eyebrow,
    headerLevel = 2,
    backHref = "/",
    backLabel = "Back to dashboard",
    query,
    onQueryChange
  }: Props = $props();

  async function dataLoader(fetch: typeof window.fetch, token: string | null, nextQuery: QueryParams) {
    if (!token) throw new Error("Not authenticated");
    const response = await mediaCommands.listMedia(fetch, token, {
      profile: "list",
      query: nextQuery
    });
    return toPagedListResult(response);
  }

  function requireToken(): string {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    return token;
  }

  async function deleteMedia(mediaId: string) {
    await mediaCommands.softDeleteMedia(mediaId, fetch, requireToken());
  }

  async function batchDeleteMedia(ids: string[]) {
    return await mediaCommands.batchDeleteMedia({ ids }, fetch, requireToken());
  }

  function handleUpload() {
    void gotoWithContext("/media/upload", {
      label: title,
      href: backHref,
      type: "list"
    });
  }
</script>

<MediaListPage
  {title}
  {hideTitle}
  {subtitle}
  {eyebrow}
  {headerLevel}
  {backHref}
  {backLabel}
  {dataLoader}
  {query}
  {onQueryChange}
  onUpload={handleUpload}
  onViewTrash={() => goto("/media/trash")}
  onDeleteMedia={deleteMedia}
  onBatchDelete={batchDeleteMedia}
/>
