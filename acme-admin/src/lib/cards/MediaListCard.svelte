<script lang="ts">
  import { AlertDialog as PoodleAlertDialog, MediaThumbnail as PoodleMediaThumbnail, formatFileSize } from "@poodle/svelte";
  import { EntityListCard, type EntityListCardBadge } from "@decodelabs/underlay/templates";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import {
    getMediaKindAccent,
    getMediaKindLabel,
    getMediaVisibilityLabel,
    MediaKind,
    MediaVisibility
  } from "@decodelabs/underlay/runtime/media";
  import type { MediaSummary } from "@api-client";

  interface Props {
    media: MediaSummary;
    selectionMode?: boolean;
    reorderMode?: boolean;
    selected?: boolean;
    onSelectionChange?: (mediaId: string, selected: boolean) => void;
    onDelete?: (mediaId: string) => void;
    onCopyId?: (mediaId: string) => void;
  }

  let {
    media,
    selectionMode = false,
    reorderMode = false,
    selected = false,
    onSelectionChange,
    onDelete,
    onCopyId
  }: Props = $props();

  let confirmDeleteOpen = $state(false);

  const title = $derived(media.title ?? media.originalFilename ?? "Untitled");
  const subtitle = $derived(
    media.title && media.originalFilename && media.originalFilename !== media.title
      ? media.originalFilename
      : null
  );
  const badges = $derived<EntityListCardBadge[]>([
    { label: getMediaKindLabel(media.kind) },
    ...(media.visibility && media.visibility !== MediaVisibility.Public
      ? [{ label: getMediaVisibilityLabel(media.visibility) }]
      : [])
  ]);
  const footerText = $derived(
    [
      media.byteSize ? formatFileSize(media.byteSize) : null,
      `Updated ${new Date(media.updatedAt).toLocaleDateString()}`
    ].filter(Boolean).join(" · ")
  );
  const menuItems = $derived([
    { value: "copy-id", label: "Copy media ID" },
    ...(onDelete
      ? [
          { value: "separator", label: "", kind: "separator" as const },
          { value: "delete", label: "Move to trash", tone: "danger" as const }
        ]
      : [])
  ]);
  const thumbnailKind = $derived.by((): "image" | "audio" | "video" | "document" | "embed" => {
    if (media.kind === MediaKind.Image) return "image";
    if (media.kind === MediaKind.Audio) return "audio";
    if (media.kind === MediaKind.Video) return "video";
    return "document";
  });

  function handleOpen(): void {
    void gotoWithContext(`/media/${media.id}`, {
      label: "Media",
      href: "/media",
      type: "list"
    });
  }

  function handleDelete(): void {
    onDelete?.(media.id);
    confirmDeleteOpen = false;
  }

  function handleContextAction(value: string): void {
    if (value === "copy-id") {
      onCopyId?.(media.id);
      return;
    }

    if (value === "delete") {
      confirmDeleteOpen = true;
    }
  }
</script>

{#snippet mediaLeading()}
  <PoodleMediaThumbnail
    kind={thumbnailKind}
    presentation="default"
    aspectRatio="square"
    ariaLabel={title}
  >
    {#if media.thumbnailUrl}
      <img
        src={media.thumbnailUrl}
        alt={media.title ?? ""}
        class="media-list-card__thumbnail-image"
      />
    {/if}
  </PoodleMediaThumbnail>
{/snippet}

<EntityListCard
  title={title}
  {subtitle}
  {reorderMode}
  selectionMode={selectionMode}
  {selected}
  badges={badges}
  footerText={footerText}
  leading={mediaLeading}
  contextMenuItems={selectionMode || reorderMode ? [] : menuItems}
  contextMenuAriaLabel="Media actions"
  contextMenuTrigger="leading"
  onSelectionChange={(nextSelected) => onSelectionChange?.(media.id, nextSelected)}
  onContextAction={handleContextAction}
  onClick={selectionMode || reorderMode ? undefined : handleOpen}
/>

{#if confirmDeleteOpen}
  <PoodleAlertDialog
    open={confirmDeleteOpen}
    title="Move media to trash"
    description={`Are you sure you want to move "${title}" to trash? You can restore it later.`}
    confirmLabel="Move to trash"
    onConfirm={handleDelete}
    onCancel={() => {
      confirmDeleteOpen = false;
    }}
    tone="danger"
  />
{/if}
