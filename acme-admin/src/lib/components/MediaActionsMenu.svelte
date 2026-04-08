<script lang="ts">
import {
  copyToClipboard,
  useToasts
} from "@decodelabs/underlay/runtime/feedback";
import {
  type NavigationContext,
} from "@decodelabs/underlay/runtime/navigation";
import {
  AlertDialog as PoodleAlertDialog,
  Button,
  Menu,
  type MenuItem
  } from "@poodle/svelte-primitives";
  import type { Snippet } from "svelte";
    import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import { goto } from "$app/navigation";
  import { mediaCommands, type MediaDetail, type MediaSummary } from "@api-client";
  import { auth } from "$lib/stores/auth";

  type MediaItem = MediaSummary | MediaDetail;

  interface Props {
    media: MediaItem;
    trigger?: Snippet;
    onSoftDeleteSuccess?: () => void;
    onRestoreSuccess?: () => void;
    onPurgeSuccess?: () => void;
    onEditRequest?: () => void;
  }

  let {
    media,
    trigger,
    onSoftDeleteSuccess,
    onRestoreSuccess,
    onPurgeSuccess,
    onEditRequest
  }: Props = $props();

  const toastStore = useToasts();

  const defaultContext = $derived<NavigationContext>({
    label: "Media",
    href: `/media/${media.id}`,
    type: "detail"
  });

  const mediaDisplayName = $derived(media.title || media.originalFilename || "Untitled");
  const isDeleted = $derived(media.deletedAt !== null);

  let softDeleteOpen = $state(false);
  let restoreOpen = $state(false);
  let purgeOpen = $state(false);

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
    void gotoWithContext(`/media/upload?replace=${mediaId}`, defaultContext);
  }

  function handlePurgeSuccess() {
    if (onPurgeSuccess) {
      onPurgeSuccess();
    } else {
      goto("/media");
    }
  }

  async function handleCopy(text: string, successMessage: string, failureMessage: string) {
    await copyToClipboard(toastStore, text, successMessage, failureMessage);
  }

  async function confirmSoftDelete() {
    try {
      await softDelete(media.id);
      softDeleteOpen = false;
      toastStore.push({ variant: "success", message: "Media soft-deleted" });
      onSoftDeleteSuccess?.();
    } catch (error) {
      console.error("Failed to soft-delete media", error);
      toastStore.push({ variant: "error", message: "Failed to soft-delete media" });
    }
  }

  async function confirmRestore() {
    try {
      await restore(media.id);
      restoreOpen = false;
      toastStore.push({ variant: "success", message: "Media restored" });
      onRestoreSuccess?.();
    } catch (error) {
      console.error("Failed to restore media", error);
      toastStore.push({ variant: "error", message: "Failed to restore media" });
    }
  }

  async function confirmPurge() {
    try {
      await purge(media.id);
      purgeOpen = false;
      toastStore.push({ variant: "success", message: "Media permanently deleted" });
      handlePurgeSuccess();
    } catch (error) {
      console.error("Failed to purge media", error);
      toastStore.push({ variant: "error", message: "Failed to permanently delete media" });
    }
  }

  const menuEntries = $derived.by(() => {
    const entries: Array<{
      key: string;
      label: string;
      tone?: "default" | "danger";
      onSelect: () => void | Promise<void>;
    } | { separator: true; key: string }> = [];

    if (onEditRequest && !isDeleted) {
      entries.push({ key: "edit", label: "Edit", onSelect: () => onEditRequest() });
    }
    if (!isDeleted) {
      entries.push({ key: "replace", label: "Replace file", onSelect: () => navigateToReplace(media.id) });
    }
    if (!isDeleted) {
      entries.push({ key: "soft-delete", label: "Soft delete", tone: "danger", onSelect: () => { softDeleteOpen = true; } });
    }
    if (isDeleted) {
      entries.push({ key: "restore", label: "Restore media", onSelect: () => { restoreOpen = true; } });
      entries.push({ key: "purge", label: "Permanently delete", tone: "danger", onSelect: () => { purgeOpen = true; } });
    }

    const copyEntries: Array<{ key: string; label: string; onSelect: () => Promise<void> }> = [
      {
        key: "copy-id",
        label: "Copy ID",
        onSelect: () => handleCopy(media.id, "Copied ID", "Failed to copy ID")
      }
    ];

    if (media.originalFilename) {
      copyEntries.push({
        key: "copy-filename",
        label: "Copy filename",
        onSelect: () => handleCopy(media.originalFilename!, "Copied filename", "Failed to copy filename")
      });
    }

    if (entries.length && copyEntries.length) {
      entries.push({ separator: true, key: "separator-copy" });
    }

    entries.push(...copyEntries);
    return entries;
  });

  const menuItems = $derived<MenuItem[]>(
    menuEntries.map((entry) =>
      "separator" in entry
        ? { value: entry.key, label: "", kind: "separator" }
        : {
            value: entry.key,
            label: entry.label,
            tone: entry.tone
          }
    )
  );

  async function handleAction(value: string) {
    const entry = menuEntries.find((item) => !("separator" in item) && item.key === value);
    if (entry && !("separator" in entry)) {
      await entry.onSelect();
    }
  }
</script>

<Menu
  items={menuItems}
  placement="bottom-end"
  ariaLabel="Media actions"
  triggerAriaLabel="Media actions"
  on:action={(event) => void handleAction(event.detail.value)}
>
  <svelte:fragment slot="trigger">
    {#if trigger}
      {@render trigger()}
    {:else}
      <Button variant="secondary">Actions</Button>
    {/if}
  </svelte:fragment>
</Menu>

<PoodleAlertDialog
  bind:open={softDeleteOpen}
  title="Soft delete media?"
  description="Soft deleting will hide this media from listings. You can restore it later from trash."
  confirmLabel="Soft delete"
  cancelLabel="Cancel"
  onConfirm={confirmSoftDelete}
  onCancel={() => (softDeleteOpen = false)}
  tone="danger"
>
  <p>Media: <strong>{mediaDisplayName}</strong></p>
</PoodleAlertDialog>

<PoodleAlertDialog
  bind:open={restoreOpen}
  title="Restore media?"
  description="This will restore the media back to the library."
  confirmLabel="Restore"
  cancelLabel="Cancel"
  onConfirm={confirmRestore}
  onCancel={() => (restoreOpen = false)}
  tone="warning"
>
  <p>Media: <strong>{mediaDisplayName}</strong></p>
</PoodleAlertDialog>

<PoodleAlertDialog
  bind:open={purgeOpen}
  title="Permanently delete media?"
  description="This will permanently delete the media and all its versions. This cannot be undone."
  confirmLabel="Delete permanently"
  cancelLabel="Cancel"
  onConfirm={confirmPurge}
  onCancel={() => (purgeOpen = false)}
  tone="danger"
>
  <p>Media: <strong>{mediaDisplayName}</strong></p>
</PoodleAlertDialog>
