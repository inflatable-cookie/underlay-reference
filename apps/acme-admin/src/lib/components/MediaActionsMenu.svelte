<script lang="ts">
  import { goto } from "$app/navigation";
  import { gotoWithContext } from "@inflatable-cookie/underlay/client/navigation";
  import { MediaActionsMenu as SharedMediaActionsMenu } from "@inflatable-cookie/underlay/templates";
  import type { MediaActionsMenuItem, TemplateSurface } from "@inflatable-cookie/underlay/templates";
  import type { NavigationContext } from "@inflatable-cookie/underlay/runtime/navigation";
  import { mediaCommands, type MediaDetail, type MediaSummary } from "@api-client";
  import { auth } from "$lib/stores/auth";

  type MediaItem = MediaSummary | MediaDetail;

  interface Props {
    media: MediaItem;
    sourceContext?: NavigationContext;
    trigger?: TemplateSurface;
    onSoftDeleteSuccess?: () => void;
    onRestoreSuccess?: () => void;
    onPurgeSuccess?: () => void;
    onEditRequest?: () => void;
  }

  let {
    media,
    sourceContext,
    trigger,
    onSoftDeleteSuccess,
    onRestoreSuccess,
    onPurgeSuccess,
    onEditRequest
  }: Props = $props();

  function requireToken(): string {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    return token;
  }

  async function softDeleteAction(item: MediaActionsMenuItem): Promise<void> {
    await mediaCommands.softDeleteMedia(item.id, window.fetch.bind(window), requireToken());
  }

  async function restoreAction(item: MediaActionsMenuItem): Promise<void> {
    await mediaCommands.restoreMedia(item.id, window.fetch.bind(window), requireToken());
  }

  async function purgeAction(item: MediaActionsMenuItem): Promise<void> {
    await mediaCommands.purgeMedia(item.id, window.fetch.bind(window), requireToken());
  }

  function handleReplaceRequest(item: MediaActionsMenuItem, context: NavigationContext): void {
    void gotoWithContext(`/media/upload?replace=${item.id}`, sourceContext ?? context);
  }

  function handlePurgeSuccess(): void {
    if (onPurgeSuccess) {
      onPurgeSuccess();
      return;
    }

    void goto("/media");
  }
</script>

<SharedMediaActionsMenu
  {media}
  {sourceContext}
  {trigger}
  {softDeleteAction}
  {restoreAction}
  {purgeAction}
  onReplaceRequest={handleReplaceRequest}
  {onSoftDeleteSuccess}
  {onRestoreSuccess}
  onPurgeSuccess={handlePurgeSuccess}
  {onEditRequest}
/>
