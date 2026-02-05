<script lang="ts">
  /**
   * MediaActionsMenu - Acme wrapper around Underlay's MediaActionsMenu component.
   *
   * This wrapper binds acme-client media commands to the generic component.
   */
  import type { Snippet } from "svelte";
  import { MediaActionsMenu as BaseMediaActionsMenu } from "@decodelabs/underlay/components";
  import { type NavigationContext } from "@decodelabs/underlay/patterns";
  import { gotoWithContext } from "@decodelabs/underlay/client";
  import { goto } from "$app/navigation";
  import { mediaCommands, type MediaDetail, type MediaSummary } from "acme-client";
  import { auth } from "$lib/stores/auth";

  type MediaItem = MediaSummary | MediaDetail;

  interface Props {
    /** Media data */
    media: MediaItem;
    /** Navigation context for actions */
    sourceContext?: NavigationContext;
    /** Optional custom trigger snippet (for ListCard usage) */
    trigger?: Snippet;
    /** Callback after successful soft delete */
    onSoftDeleteSuccess?: () => void;
    /** Callback after successful restore */
    onRestoreSuccess?: () => void;
    /** Callback after successful purge */
    onPurgeSuccess?: () => void;
    /** Callback when edit is requested */
    onEditRequest?: () => void;
  }

  let {
    media,
    sourceContext,
    trigger,
    onSoftDeleteSuccess,
    onRestoreSuccess,
    onPurgeSuccess,
    onEditRequest,
  }: Props = $props();

  const defaultContext = $derived<NavigationContext>({
    label: "Media",
    href: `/media/${media.id}`,
    type: "detail",
  });

  // Bind acme-client commands with auth token
  async function softDelete(mediaId: string) {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    await mediaCommands.softDeleteMedia(mediaId, window.fetch.bind(window), token);
  }

  async function restore(mediaId: string) {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    await mediaCommands.restoreMedia(mediaId, window.fetch.bind(window), token);
  }

  async function purge(mediaId: string) {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    await mediaCommands.purgeMedia(mediaId, window.fetch.bind(window), token);
  }

  function navigateToReplace(mediaId: string) {
    void gotoWithContext(`/media/upload?replace=${mediaId}`, sourceContext ?? defaultContext);
  }

  // Handle purge success - navigate to media list if no custom handler
  function handlePurgeSuccess() {
    if (onPurgeSuccess) {
      onPurgeSuccess();
    } else {
      goto("/media");
    }
  }
</script>

<BaseMediaActionsMenu
  {media}
  {sourceContext}
  {trigger}
  {softDelete}
  {restore}
  {purge}
  {navigateToReplace}
  {onSoftDeleteSuccess}
  {onRestoreSuccess}
  onPurgeSuccess={handlePurgeSuccess}
  {onEditRequest}
/>
