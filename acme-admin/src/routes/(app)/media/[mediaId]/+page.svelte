<script lang="ts">
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";
  import {
    mediaCommands,
    type MediaDetail,
    MediaKind,
    MediaVersionState,
    getMediaKindLabel,
    getMediaVersionStateLabel
  } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import { useAuthenticatedData, PageHeader, useToasts } from "@decodelabs/underlay/patterns";
  import { Button, PageLoading, FormError, ConfirmAction, Badge, Field, TextInput } from "@decodelabs/underlay/components";
  import { gotoWithContext } from "@decodelabs/underlay/client";
  import Pencil from "lucide-svelte/icons/pencil";
  import Upload from "lucide-svelte/icons/upload";
  import Image from "lucide-svelte/icons/image";
  import FileText from "lucide-svelte/icons/file-text";
  import Film from "lucide-svelte/icons/film";
  import Music from "lucide-svelte/icons/music";
  import FileIcon from "lucide-svelte/icons/file";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  const toastStore = useToasts();

  // Form state for editing
  let isEditing = $state(false);
  let editTitle = $state("");
  let saving = $state(false);

  // Fetch media data
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const item = await mediaCommands.getMedia(data.mediaId, fetch, token);
      return { item };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { item: null as MediaDetail | null }
    }
  );

  // Trigger fetch when auth is ready
  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  const item = $derived(pageData.data?.item);
  const currentVersion = $derived(item?.currentVersion);

  function startEditing() {
    if (!item) return;
    editTitle = item.title ?? "";
    isEditing = true;
  }

  function cancelEditing() {
    isEditing = false;
  }

  async function saveChanges() {
    if (!item) return;

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    saving = true;

    try {
      await mediaCommands.updateMedia(
        item.id,
        { title: editTitle.trim() || null },
        fetch,
        token
      );
      toastStore.push({ variant: "success", message: "Media updated" });
      isEditing = false;
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to update media";
      toastStore.push({ variant: "error", message });
    } finally {
      saving = false;
    }
  }

  async function handleDelete() {
    if (!item) return;

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await mediaCommands.softDeleteMedia(item.id, fetch, token);
      toastStore.push({ variant: "success", message: "Media moved to trash" });
      await goto("/media");
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete media";
      toastStore.push({ variant: "error", message });
    }
  }

  function getKindIcon(kind: string) {
    switch (kind) {
      case MediaKind.Image:
        return Image;
      case MediaKind.Video:
        return Film;
      case MediaKind.Audio:
        return Music;
      case MediaKind.Document:
      case MediaKind.Pdf:
        return FileText;
      default:
        return FileIcon;
    }
  }

  function formatFileSize(bytes: number | null | undefined): string {
    if (bytes == null || bytes === 0) return "—";
    const units = ["B", "KB", "MB", "GB"];
    let size = bytes;
    let unitIndex = 0;
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    return `${size.toFixed(unitIndex > 0 ? 1 : 0)} ${units[unitIndex]}`;
  }

  type BadgeVariant = "default" | "success" | "warning" | "danger" | "info" | "muted";

  function getVersionStateVariant(state: string): BadgeVariant {
    switch (state) {
      case MediaVersionState.Ready:
        return "success";
      case MediaVersionState.Uploading:
        return "info";
      case MediaVersionState.Failed:
        return "danger";
      case MediaVersionState.Purging:
        return "warning";
      default:
        return "muted";
    }
  }
</script>

{#if pageData.loading}
  <PageLoading message="Loading media..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if item}
  {@const KindIcon = getKindIcon(item.kind)}
  <PageHeader
    title={item.title ?? item.originalFilename ?? "Untitled"}
    backHref="/media"
    backLabel="Back to media"
  >
    {#snippet actions()}
      {#if !isEditing}
        <Button type="button" variant="subtle" onclick={startEditing}>
          <Pencil size={16} />
          Edit
        </Button>
        <Button
          type="button"
          variant="subtle"
          onclick={() =>
            void gotoWithContext(`/media/upload?replace=${item.id}`, {
              label: item.title ?? item.originalFilename ?? "Media",
              href: `/media/${item.id}`,
              type: "detail"
            })}
        >
          <Upload size={16} />
          Replace
        </Button>
        <ConfirmAction
          title="Delete Media"
          description={`Are you sure you want to delete "${item.title ?? item.originalFilename}"? It will be moved to trash.`}
          confirmLabel="Delete"
          triggerLabel="Delete"
          triggerVariant="danger"
          onConfirm={handleDelete}
        />
      {/if}
    {/snippet}
  </PageHeader>

  <div class="detail-grid">
    <!-- Preview -->
    <section class="preview-section">
      {#if currentVersion?.url && item.kind === MediaKind.Image}
        <img
          src={currentVersion.url}
          alt={item.title ?? item.originalFilename ?? "Media preview"}
          class="preview-image"
        />
      {:else}
        <div class="preview-placeholder">
          <KindIcon size={48} />
          <span>{getMediaKindLabel(item.kind)}</span>
        </div>
      {/if}
    </section>

    <!-- Details -->
    <section class="detail-section">
      <h2>{isEditing ? "Edit Details" : "Details"}</h2>

      {#if isEditing}
        <div class="edit-form">
          <Field label="Title">
            <TextInput
              bind:value={editTitle}
              placeholder="Enter title"
              disabled={saving}
            />
          </Field>
          <div class="edit-actions">
            <Button type="button" variant="subtle" onclick={cancelEditing} disabled={saving}>
              Cancel
            </Button>
            <Button type="button" variant="primary" onclick={saveChanges} disabled={saving}>
              {saving ? "Saving..." : "Save"}
            </Button>
          </div>
        </div>
      {:else}
        <dl class="detail-list">
          <div class="detail-item">
            <dt>Type</dt>
            <dd><Badge variant="info">{getMediaKindLabel(item.kind)}</Badge></dd>
          </div>
          <div class="detail-item">
            <dt>Filename</dt>
            <dd><code>{item.originalFilename ?? "—"}</code></dd>
          </div>
          {#if currentVersion}
            <div class="detail-item">
              <dt>MIME Type</dt>
              <dd><code>{currentVersion.mimeType ?? "—"}</code></dd>
            </div>
            <div class="detail-item">
              <dt>Size</dt>
              <dd>{formatFileSize(currentVersion.byteSize)}</dd>
            </div>
            <div class="detail-item">
              <dt>Version State</dt>
              <dd>
                <Badge variant={getVersionStateVariant(currentVersion.state)} size="sm">
                  {getMediaVersionStateLabel(currentVersion.state)}
                </Badge>
              </dd>
            </div>
          {/if}
          <div class="detail-item">
            <dt>Visibility</dt>
            <dd>{item.visibility}</dd>
          </div>
        </dl>
      {/if}
    </section>

    <!-- Renditions -->
    {#if currentVersion?.renditions && currentVersion.renditions.length > 0}
      <section class="detail-section">
        <h2>Renditions</h2>
        <div class="renditions-grid">
          {#each currentVersion.renditions as rendition}
            <div class="rendition-card">
              {#if rendition.url && rendition.mimeType?.startsWith("image/")}
                <img src={rendition.url} alt={rendition.kind} class="rendition-preview" />
              {:else}
                <div class="rendition-placeholder">
                  <FileIcon size={24} />
                </div>
              {/if}
              <div class="rendition-info">
                <span class="rendition-kind">{rendition.kind}</span>
                <span class="rendition-size">
                  {rendition.width && rendition.height
                    ? `${rendition.width}×${rendition.height}`
                    : formatFileSize(rendition.byteSize)}
                </span>
              </div>
            </div>
          {/each}
        </div>
      </section>
    {/if}

    <!-- Metadata -->
    <section class="detail-section">
      <h2>Metadata</h2>
      <dl class="detail-list">
        <div class="detail-item">
          <dt>ID</dt>
          <dd><code>{item.id}</code></dd>
        </div>
        {#if currentVersion}
          <div class="detail-item">
            <dt>Version ID</dt>
            <dd><code>{currentVersion.id}</code></dd>
          </div>
        {/if}
        <div class="detail-item">
          <dt>Created</dt>
          <dd>{new Date(item.createdAt).toLocaleString()}</dd>
        </div>
        <div class="detail-item">
          <dt>Updated</dt>
          <dd>{new Date(item.updatedAt).toLocaleString()}</dd>
        </div>
      </dl>
    </section>
  </div>
{:else}
  <FormError message="Media not found" />
{/if}

<style>
  .detail-grid {
    display: grid;
    gap: 2rem;
    margin-top: 1.5rem;
  }

  .preview-section {
    background: var(--bg-surface, #fff);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 0.5rem;
    padding: 1.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .preview-image {
    max-width: 100%;
    max-height: 400px;
    object-fit: contain;
    border-radius: 0.25rem;
  }

  .preview-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 3rem;
    color: var(--text-secondary, #6b7280);
  }

  .renditions-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 1rem;
  }

  .rendition-card {
    background: var(--bg-muted, #f3f4f6);
    border-radius: 0.5rem;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .rendition-preview {
    width: 100%;
    height: 80px;
    object-fit: cover;
  }

  .rendition-placeholder {
    width: 100%;
    height: 80px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary, #6b7280);
  }

  .rendition-info {
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .rendition-kind {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-primary, #111827);
    text-transform: capitalize;
  }

  .rendition-size {
    font-size: 0.7rem;
    color: var(--text-secondary, #6b7280);
  }

  .detail-section {
    background: var(--bg-surface, #fff);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 0.5rem;
    padding: 1.5rem;
  }

  .detail-section h2 {
    margin: 0 0 1rem;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary, #111827);
  }

  .detail-list {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
    margin: 0;
  }

  .detail-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .detail-item dt {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-secondary, #6b7280);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .detail-item dd {
    margin: 0;
    font-size: 0.875rem;
    color: var(--text-primary, #111827);
  }

  code {
    font-family: monospace;
    font-size: 0.8em;
    background: var(--bg-muted, #f3f4f6);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }

  .edit-form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .edit-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 0.5rem;
  }
</style>
