<script lang="ts">
  import { goto } from "$app/navigation";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import { EntityActionsMenu } from "@decodelabs/underlay/templates";
  import type { TemplateSurface } from "@decodelabs/underlay/templates";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";
  import type { NavigationContext } from "@decodelabs/underlay/runtime/navigation";
  import { AlertDialog } from "@poodle/svelte";
  import {
    getMediaDisplayName,
    isMediaDeleted,
    mediaCommands,
    type MediaDetail,
    type MediaSummary
  } from "@api-client";
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

  const toastStore = useToasts();
  const isDeleted = $derived(isMediaDeleted(media));
  const mediaDisplayName = $derived(getMediaDisplayName(media));
  const defaultContext = $derived<NavigationContext>({
    label: "Media",
    href: `/media/${media.id}`,
    type: "detail"
  });

  let restoreOpen = $state(false);
  let restoreBusy = $state(false);

  function requireToken(): string {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    return token;
  }

  async function executeSoftDelete(): Promise<void> {
    try {
      await mediaCommands.softDeleteMedia(media.id, window.fetch.bind(window), requireToken());
    } catch {
      throw new Error("Failed to move media to trash");
    }
    toastStore.push({ variant: "success", message: "Media moved to trash" });
  }

  async function executePurge(): Promise<void> {
    try {
      await mediaCommands.purgeMedia(media.id, window.fetch.bind(window), requireToken());
    } catch {
      throw new Error("Failed to permanently delete media");
    }
    toastStore.push({ variant: "success", message: "Media permanently deleted" });
  }

  async function confirmRestore(): Promise<void> {
    if (restoreBusy) return;

    restoreBusy = true;
    try {
      await mediaCommands.restoreMedia(media.id, window.fetch.bind(window), requireToken());
      restoreOpen = false;
      toastStore.push({ variant: "success", message: "Media restored" });
      onRestoreSuccess?.();
    } catch {
      toastStore.push({ variant: "error", message: "Failed to restore media" });
    } finally {
      restoreBusy = false;
    }
  }

  function handlePurgeSuccess(): void {
    if (onPurgeSuccess) {
      onPurgeSuccess();
      return;
    }

    void goto("/media");
  }

  const copies = $derived([
    {
      label: "Copy ID",
      text: media.id,
      successMessage: "Copied ID",
      failureMessage: "Failed to copy ID"
    },
    ...(media.originalFilename
      ? [
          {
            label: "Copy filename",
            text: media.originalFilename,
            successMessage: "Copied filename",
            failureMessage: "Failed to copy filename"
          }
        ]
      : [])
  ]);

  const customActions = $derived(
    isDeleted
      ? [
          {
            label: "Restore media",
            disabled: restoreBusy,
            onSelect: () => {
              restoreOpen = true;
            }
          }
        ]
      : [
          {
            label: "Replace file",
            onSelect: () => {
              void gotoWithContext(
                `/media/upload?replace=${media.id}`,
                sourceContext ?? defaultContext
              );
            }
          }
        ]
  );

  const deleteConfig = $derived(
    isDeleted
      ? {
          entityLabel: mediaDisplayName,
          title: "Permanently delete media?",
          description:
            "This removes the media and all versions permanently. This cannot be undone.",
          confirmLabel: "Delete permanently",
          execute: executePurge
        }
      : {
          entityLabel: mediaDisplayName,
          title: "Move media to trash?",
          description:
            "This hides the media from the main library. You can restore it later from trash.",
          confirmLabel: "Move to trash",
          execute: executeSoftDelete
        }
  );
</script>

<EntityActionsMenu
  {trigger}
  {copies}
  {customActions}
  {deleteConfig}
  onEdit={isDeleted ? undefined : onEditRequest}
  triggerAriaLabel="Media actions"
  triggerTooltip="Actions"
  onDeleteSuccess={isDeleted ? handlePurgeSuccess : onSoftDeleteSuccess}
/>

{#if restoreOpen || restoreBusy}
  <AlertDialog
    bind:open={restoreOpen}
    title="Restore media?"
    description="This returns the media to the active library."
    confirmLabel={restoreBusy ? "Working..." : "Restore"}
    cancelLabel="Cancel"
    tone="warning"
    onConfirm={confirmRestore}
    onCancel={() => {
      if (!restoreBusy) {
        restoreOpen = false;
      }
    }}
  >
    <p>Media: <strong>{mediaDisplayName}</strong></p>
  </AlertDialog>
{/if}
