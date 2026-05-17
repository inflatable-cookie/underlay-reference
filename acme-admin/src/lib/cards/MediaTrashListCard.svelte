<script lang="ts">
  import {
    AlertDialog as PoodleAlertDialog,
    Button as PoodleButton,
    MediaThumbnail as PoodleMediaThumbnail,
    formatFileSize
  } from "@poodle/svelte";
  import { EntityListCard, type EntityListCardBadge } from "@decodelabs/underlay/templates";
  import { getMediaKindLabel } from "@decodelabs/underlay/runtime/media";
  import type { MediaSummary } from "@api-client";
  import RotateCcw from "lucide-svelte/icons/rotate-ccw";
  import Trash2 from "lucide-svelte/icons/trash-2";

  interface Props {
    media: MediaSummary;
    onRestore?: (mediaId: string) => void;
    onPurge?: (mediaId: string) => void;
  }

  let { media, onRestore, onPurge }: Props = $props();

  let confirmPurgeOpen = $state(false);
  let failedThumbnailUrl = $state<string | null>(null);

  const title = $derived(media.title ?? media.originalFilename ?? "Untitled");
  const rawThumbnailUrl = $derived(media.thumbnailUrl?.trim() || null);
  const previewImageUrl = $derived(
    rawThumbnailUrl && rawThumbnailUrl !== failedThumbnailUrl ? rawThumbnailUrl : null
  );
  const badges = $derived<EntityListCardBadge[]>([
    { label: getMediaKindLabel(media.kind), tone: "neutral", appearance: "badge", size: "lg" },
    { label: "Deleted", tone: "danger", appearance: "badge", size: "lg" }
  ]);
  const footerText = $derived(
    [
      media.byteSize ? formatFileSize(media.byteSize) : null,
      media.deletedAt ? `Deleted ${new Date(media.deletedAt).toLocaleDateString()}` : null
    ]
      .filter(Boolean)
      .join(" · ")
  );

  function toPoodleMediaKind(kind: string): "image" | "audio" | "video" | "document" | "embed" {
    if (kind === "image") return "image";
    if (kind === "audio") return "audio";
    if (kind === "video") return "video";
    return "document";
  }
  const leadingIcon = $derived.by((): string => {
    if (media.kind === "image") return "image";
    if (media.kind === "audio") return "music";
    if (media.kind === "video") return "video";
    return "file-text";
  });

  function handleRestore(): void {
    onRestore?.(media.id);
  }

  function handlePurge(): void {
    onPurge?.(media.id);
    confirmPurgeOpen = false;
  }
</script>

{#snippet mediaLeading()}
  <PoodleMediaThumbnail
    kind={toPoodleMediaKind(media.kind)}
    presentation="compact"
    aspectRatio="square"
    ariaLabel={title}
  >
    <img
      src={previewImageUrl}
      alt={media.title ?? ""}
      class="media-trash-card__thumbnail-image"
      onerror={() => {
        failedThumbnailUrl = rawThumbnailUrl;
      }}
    />
  </PoodleMediaThumbnail>
{/snippet}

{#snippet mediaFooter()}
  <div class="media-trash-card__actions">
    <PoodleButton type="button" variant="ghost" size="sm" onClick={handleRestore}>
      <RotateCcw size={14} />
      Restore
    </PoodleButton>
    <PoodleButton
      type="button"
      variant="ghost"
      tone="danger"
      size="sm"
      onClick={() => (confirmPurgeOpen = true)}
    >
      <Trash2 size={14} />
      Delete
    </PoodleButton>
  </div>
{/snippet}

<EntityListCard
  title={title}
  href={`/media/${media.id}`}
  accentColor="#64748b"
  {badges}
  footerText={footerText}
  leading={previewImageUrl ? mediaLeading : undefined}
  {leadingIcon}
  footer={mediaFooter}
/>

{#if confirmPurgeOpen}
  <PoodleAlertDialog
    open={confirmPurgeOpen}
    title="Permanently Delete"
    description={`Are you sure you want to permanently delete "${title}"? This action cannot be undone.`}
    confirmLabel="Delete Forever"
    onConfirm={handlePurge}
    onCancel={() => {
      confirmPurgeOpen = false;
    }}
    tone="danger"
  />
{/if}

<style>
  .media-trash-card__thumbnail-image {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .media-trash-card__actions {
    display: flex;
    gap: 0.5rem;
  }
</style>
