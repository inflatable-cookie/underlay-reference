<script lang="ts">
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";
  import { env } from "$env/dynamic/public";
  import {
    PageHeader,
    getBackButtonInfo,
    useToasts,
    useAuthenticatedData,
    FormDialog,
    getMediaKindLabel,
    getMediaKindAccent,
    getMediaVisibilityLabel,
    getMediaVersionStateLabel,
    getMediaVersionStateAccent,
    formatFileSize
  } from "@decodelabs/underlay/patterns";
  import {
    AlertDialog,
    Button,
    Code,
    DetailsCard,
    DetailsItem,
    DetailsSection,
    Field,
    FormError,
    InlineListCard,
    InlineListItem,
    PageLoading,
    Pill,
    Select,
    TabsRoot,
    TabsList,
    TabsTrigger,
    TabsContent,
    TextInput,
    TimeAgo
  } from "@decodelabs/underlay/components";
  import {
    mediaCommands,
    type MediaDetail,
    type MediaVersion,
    type MediaUsage,
    MediaKind,
    MediaVisibility,
    MediaVersionState
  } from "acme-client";
  import { gotoWithContext } from "@decodelabs/underlay/client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import Check from "lucide-svelte/icons/check";
  import Plus from "lucide-svelte/icons/plus";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import { browser } from "$app/environment";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  const toastStore = useToasts();
  const mediaId = data.mediaId;

  // Load media detail, versions, and usages
  const pageData = useAuthenticatedData(
    async (fetchFn, token) => {
      const [media, versions, usages] = await Promise.all([
        mediaCommands.getMedia(mediaId, fetchFn, token),
        mediaCommands.listVersions(mediaId, fetchFn, token),
        mediaCommands.listUsages(mediaId, fetchFn, token)
      ]);
      return { media, versions, usages };
    },
    { getToken: () => auth.getToken() }
  );

  // Trigger fetch when auth is ready
  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  const media = $derived(pageData.data?.media);
  const versions = $derived(pageData.data?.versions ?? []);
  const usages = $derived(pageData.data?.usages ?? []);

  let activeTab = $state("details");
  const usageCount = $derived(usages.length);

  const backInfo = getBackButtonInfo("Back to media", "/media");

  // Edit dialog state
  let editDialogOpen = $state(false);
  let editDialogError = $state<string | null>(null);
  let editDialogSubmitting = $state(false);
  let editTitle = $state("");
  let editFilename = $state("");
  let editVisibility = $state<string>(MediaVisibility.Public);

  function openEditDialog() {
    if (!media) return;
    editTitle = media.title || "";
    editFilename = media.originalFilename || "";
    editVisibility = media.visibility;
    editDialogError = null;
    editDialogOpen = true;
  }

  function closeEditDialog() {
    editDialogOpen = false;
    editDialogError = null;
    editDialogSubmitting = false;
  }

  async function handleEditSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!browser || !media) return;

    const token = auth.getToken();
    if (!token) {
      editDialogError = "Not authenticated";
      return;
    }

    editDialogSubmitting = true;
    editDialogError = null;

    try {
      await mediaCommands.updateMedia(
        media.id,
        {
          title: editTitle || null,
          originalFilename: editFilename || null,
          visibility: editVisibility as MediaVisibility
        },
        window.fetch.bind(window),
        token
      );
      toastStore.push({ variant: "success", message: "Media updated" });
      closeEditDialog();
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to update media";
      editDialogError = message;
    } finally {
      editDialogSubmitting = false;
    }
  }

  // Version actions state
  let activateDialogOpen = $state(false);
  let deleteDialogOpen = $state(false);
  let selectedVersion = $state<MediaVersion | null>(null);

  function requestActivate(version: MediaVersion) {
    selectedVersion = version;
    activateDialogOpen = true;
  }

  function requestDelete(version: MediaVersion) {
    selectedVersion = version;
    deleteDialogOpen = true;
  }

  async function confirmActivate() {
    if (!browser || !media || !selectedVersion) return;

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await mediaCommands.activateVersion(
        media.id,
        selectedVersion.id,
        window.fetch.bind(window),
        token
      );
      toastStore.push({ variant: "success", message: "Version activated" });
      activateDialogOpen = false;
      selectedVersion = null;
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to activate version";
      toastStore.push({ variant: "error", message });
    }
  }

  async function confirmDelete() {
    if (!browser || !media || !selectedVersion) return;

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await mediaCommands.deleteVersion(
        media.id,
        selectedVersion.id,
        window.fetch.bind(window),
        token
      );
      toastStore.push({ variant: "success", message: "Version deleted" });
      deleteDialogOpen = false;
      selectedVersion = null;
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete version";
      toastStore.push({ variant: "error", message });
    }
  }

  function cancelActivate() {
    activateDialogOpen = false;
    selectedVersion = null;
  }

  function cancelDelete() {
    deleteDialogOpen = false;
    selectedVersion = null;
  }

  // Soft delete
  async function handleSoftDelete() {
    if (!browser || !media) return;

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await mediaCommands.softDeleteMedia(
        media.id,
        window.fetch.bind(window),
        token
      );
      toastStore.push({ variant: "success", message: "Media moved to trash" });
      await goto("/media");
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete media";
      toastStore.push({ variant: "error", message });
    }
  }

  // Restore
  async function handleRestore() {
    if (!browser || !media) return;

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await mediaCommands.restoreMedia(
        media.id,
        window.fetch.bind(window),
        token
      );
      toastStore.push({ variant: "success", message: "Media restored" });
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to restore media";
      toastStore.push({ variant: "error", message });
    }
  }

  function isCurrentVersion(version: MediaVersion): boolean {
    return media?.currentVersionId === version.id;
  }

  function canActivateVersion(version: MediaVersion): boolean {
    return !isCurrentVersion(version) && version.state === MediaVersionState.Ready;
  }

  function canDeleteVersion(version: MediaVersion): boolean {
    return !isCurrentVersion(version);
  }

  /** Get the URL for viewing/downloading media */
  function getMediaUrl(mediaId: string, restricted: boolean): string {
    const base = env.PUBLIC_API_URL?.replace(/\/$/, "") ?? "";
    if (restricted) {
      return `${base}/v1/media/${encodeURIComponent(mediaId)}/download`;
    }
    return `${base}/v1/media/${encodeURIComponent(mediaId)}`;
  }

  /** Check if media can be previewed in browser */
  function canPreview(kind: string, mimeType: string | null): boolean {
    if (kind === MediaKind.Image) return true;
    if (kind === MediaKind.Pdf) return true;
    return false;
  }

  /** Check if media is an image */
  function isImage(kind: string): boolean {
    return kind === MediaKind.Image;
  }

  /** Check if media is a PDF */
  function isPdf(kind: string): boolean {
    return kind === MediaKind.Pdf;
  }

  // Derived media URL
  const mediaUrl = $derived(
    media ? getMediaUrl(media.id, media.visibility === MediaVisibility.Restricted) : null
  );

  const showPreviewTab = $derived(
    media && media.currentVersion?.state === MediaVersionState.Ready &&
    canPreview(media.kind, media.currentVersion?.mimeType ?? null)
  );
</script>

{#if pageData.loading}
  <PageLoading message="Loading media..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if media}
  <PageHeader
    title={media.title || media.originalFilename || "Untitled"}
    backHref={backInfo.href}
    backLabel={backInfo.label}
    backIsContextual={backInfo.isContextual ?? false}
    bannerMessage={media.deletedAt ? "This media has been soft-deleted." : undefined}
  >
    <p>
      <strong>ID:</strong> <Code>{media.id}</Code>
      <span class="header-separator">·</span>
      <Pill accent={getMediaKindAccent(media.kind)}>{getMediaKindLabel(media.kind)}</Pill>
      <span class="header-separator">·</span>
      <Pill accent={media.visibility === MediaVisibility.Restricted ? "#f59e0b" : "#3b82f6"}>
        {getMediaVisibilityLabel(media.visibility)}
      </Pill>
      {#if media.deletedAt}
        <span class="header-separator">·</span>
        <Pill accent="#ef4444">Deleted</Pill>
      {/if}
    </p>

    {#snippet actions()}
      {#if media.deletedAt}
        <Button type="button" variant="primary" onclick={handleRestore}>
          Restore
        </Button>
      {:else}
        <Button type="button" variant="subtle" onclick={openEditDialog}>
          Edit
        </Button>
        <Button
          type="button"
          variant="subtle"
          onclick={() =>
            void gotoWithContext(`/media/upload?replace=${media.id}`, {
              label: media.title || media.originalFilename || "Media",
              href: `/media/${media.id}`,
              type: "detail"
            })}
        >
          Replace
        </Button>
        <Button type="button" variant="danger" onclick={handleSoftDelete}>
          Delete
        </Button>
      {/if}
    {/snippet}
  </PageHeader>

  <TabsRoot bind:value={activeTab} variant="boxed" size="sm" historyKey="tab">
    <TabsList>
      <TabsTrigger value="details">Details</TabsTrigger>
      {#if showPreviewTab}
        <TabsTrigger value="preview">Preview</TabsTrigger>
      {/if}
      <TabsTrigger value="usage" count={usageCount}>Usage</TabsTrigger>
    </TabsList>

    <TabsContent value="details">
      <div class="underlay-details-content">
        <DetailsCard>
          <DetailsSection legend="File Details">
            <DetailsItem label="Original Filename" value={media.originalFilename} />
            {#if media.currentVersion}
              <DetailsItem label="File Size" value={formatFileSize(media.currentVersion.byteSize)} />
              <DetailsItem label="MIME Type">
                <Code>{media.currentVersion.mimeType ?? "—"}</Code>
              </DetailsItem>
            {/if}
          </DetailsSection>

          <DetailsSection legend="Timestamps">
            <DetailsItem label="Created">
              <TimeAgo date={media.createdAt} short />
            </DetailsItem>
            <DetailsItem label="Last Updated">
              <TimeAgo date={media.updatedAt} short />
            </DetailsItem>
            {#if media.deletedAt}
              <DetailsItem label="Deleted">
                <span class="deleted-date"><TimeAgo date={media.deletedAt} short /></span>
              </DetailsItem>
            {/if}
          </DetailsSection>
        </DetailsCard>

        <!-- Versions -->
        <InlineListCard
          title="Versions"
          emptyMessage="No versions uploaded yet."
          hasItems={versions.length > 0}
        >
          {#snippet action()}
            <Button
              type="button"
              variant="primary"
              size="icon-sm"
              onclick={() => goto(`/media/upload?replace=${media.id}`)}
              aria-label="Upload new version"
            >
              <Plus size={14} />
            </Button>
          {/snippet}
          {#each versions as version (version.id)}
            <InlineListItem
              label={version.sha256 ?? "No hash"}
              accent={getMediaVersionStateAccent(version.state)}
            >
              {#snippet sublabelContent()}
                {formatFileSize(version.byteSize)} · <Code>{version.mimeType ?? "Unknown type"}</Code> · <TimeAgo date={version.createdAt} short />
              {/snippet}
              {#snippet trailing()}
                <Pill accent={getMediaVersionStateAccent(version.state)}>
                  {getMediaVersionStateLabel(version.state)}
                </Pill>
                {#if isCurrentVersion(version)}
                  <Pill accent="#3b82f6">Current</Pill>
                {/if}
              {/snippet}
              {#snippet actions()}
                <button
                  type="button"
                  onclick={() => requestActivate(version)}
                  disabled={!canActivateVersion(version)}
                  aria-label="Activate version"
                >
                  <Check size={14} />
                </button>
                <button
                  type="button"
                  onclick={() => requestDelete(version)}
                  disabled={!canDeleteVersion(version)}
                  aria-label="Delete version"
                >
                  <Trash2 size={14} />
                </button>
              {/snippet}
            </InlineListItem>
          {/each}
        </InlineListCard>

        <!-- Renditions -->
        {#if media.currentVersion?.renditions && media.currentVersion.renditions.length > 0}
          <section class="renditions-section span-full">
            <h3>Renditions</h3>
            <div class="renditions-grid">
              {#each media.currentVersion.renditions as rendition}
                <div class="rendition-card">
                  {#if rendition.url && rendition.mimeType?.startsWith("image/")}
                    <img src={rendition.url} alt={rendition.kind} class="rendition-preview" />
                  {:else}
                    <div class="rendition-placeholder">
                      <span>No preview</span>
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
      </div>
    </TabsContent>

    {#if showPreviewTab}
      <TabsContent value="preview">
        <div class="media-preview-container">
          {#if mediaUrl}
            {#if isImage(media.kind)}
              <img
                src={mediaUrl}
                alt={media.title || media.originalFilename || "Media preview"}
                class="media-preview-image"
              />
            {:else if isPdf(media.kind)}
              <iframe
                src={mediaUrl}
                title={media.title || media.originalFilename || "PDF preview"}
                class="media-preview-pdf"
              ></iframe>
            {/if}
          {/if}
        </div>
      </TabsContent>
    {/if}

    <TabsContent value="usage">
      <div class="underlay-details-content">
        {#if usages.length === 0}
          <div class="empty-state">
            <p>This media is not used anywhere yet.</p>
          </div>
        {:else}
          <InlineListCard
            title="Usages"
            hasItems={true}
          >
            {#each usages as usage}
              <InlineListItem
                label={usage.usedByType}
                accent="#6366f1"
              >
                {#snippet sublabelContent()}
                  <Code>{usage.usedById}</Code>
                  {#if usage.field}
                    <span class="usage-field"> · {usage.field}</span>
                  {/if}
                {/snippet}
              </InlineListItem>
            {/each}
          </InlineListCard>
        {/if}
      </div>
    </TabsContent>
  </TabsRoot>

  <!-- Edit Dialog -->
  <FormDialog
    bind:open={editDialogOpen}
    title="Edit Media"
    subtitle={media.originalFilename ?? undefined}
    error={editDialogError}
    submitting={editDialogSubmitting}
    onCancel={closeEditDialog}
  >
    {#snippet children(submitting)}
      <form onsubmit={handleEditSubmit}>
        <div class="form-fields">
          <Field label="Title" forId="edit-title">
            <TextInput
              id="edit-title"
              name="title"
              bind:value={editTitle}
              placeholder="Enter a title for this media"
              disabled={submitting}
            />
          </Field>

          <Field label="Filename" forId="edit-filename" hint="The filename shown when downloading">
            <TextInput
              id="edit-filename"
              name="filename"
              bind:value={editFilename}
              placeholder="e.g. document.pdf"
              disabled={submitting}
            />
          </Field>

          <Field label="Visibility" forId="edit-visibility">
            <Select id="edit-visibility" name="visibility" bind:value={editVisibility} disabled={submitting}>
              <option value={MediaVisibility.Public}>Public - accessible without login</option>
              <option value={MediaVisibility.Restricted}>Restricted - requires authentication</option>
            </Select>
          </Field>
        </div>

        <div class="form-actions">
          <Button variant="secondary" onclick={closeEditDialog} disabled={submitting}>
            Cancel
          </Button>
          <Button type="submit" variant="primary" disabled={submitting}>
            {submitting ? "Saving..." : "Save"}
          </Button>
        </div>
      </form>
    {/snippet}
  </FormDialog>

  <!-- Activate Version Dialog -->
  <AlertDialog
    bind:open={activateDialogOpen}
    showTrigger={false}
    title="Activate version?"
    description="This will set this version as the current active version for this media item."
    confirmLabel="Activate"
    cancelLabel="Cancel"
    onConfirm={confirmActivate}
    onCancel={cancelActivate}
  >
    {#if selectedVersion}
      <p>
        Version: <Code>{selectedVersion.sha256?.slice(0, 16) ?? selectedVersion.id}...</Code>
      </p>
    {/if}
  </AlertDialog>

  <!-- Delete Version Dialog -->
  <AlertDialog
    bind:open={deleteDialogOpen}
    showTrigger={false}
    title="Delete version?"
    description="This will permanently delete this version and its stored file. This action cannot be undone."
    confirmLabel="Delete"
    cancelLabel="Cancel"
    onConfirm={confirmDelete}
    onCancel={cancelDelete}
  >
    {#if selectedVersion}
      <p>
        Version: <Code>{selectedVersion.sha256?.slice(0, 16) ?? selectedVersion.id}...</Code>
      </p>
    {/if}
  </AlertDialog>
{/if}

<style>
  .header-separator {
    color: var(--admin-color-text-muted, #9ca3af);
    margin: 0 0.5rem;
  }

  .deleted-date {
    color: var(--color-danger, #ef4444);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 3rem;
    text-align: center;
    color: var(--admin-color-text-muted);
  }

  .usage-field {
    color: var(--admin-color-text-muted);
  }

  .form-fields {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
  }

  /* Full preview tab */
  .media-preview-container {
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding: 1.5rem;
    background: var(--admin-color-surface-muted, rgba(255, 255, 255, 0.02));
    border-radius: 0.5rem;
    min-height: 400px;
  }

  .media-preview-image {
    max-width: 100%;
    max-height: 80vh;
    object-fit: contain;
    border-radius: 0.25rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .media-preview-pdf {
    width: 100%;
    height: 80vh;
    border: none;
    border-radius: 0.25rem;
    background: white;
  }

  /* Renditions section */
  .renditions-section {
    background: var(--admin-color-surface-muted, rgba(255, 255, 255, 0.02));
    border: 1px solid var(--admin-color-border-subtle, rgba(148, 163, 184, 0.25));
    border-radius: var(--underlay-radius-lg, 1rem);
    padding: 1rem;
  }

  .renditions-section h3 {
    margin: 0 0 1rem;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--admin-color-text, #e5e7eb);
  }

  .renditions-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 1rem;
  }

  .rendition-card {
    background: var(--admin-color-surface-inset, #1e293b);
    border-radius: var(--underlay-radius-md, 0.5rem);
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
    color: var(--admin-color-text-muted, #9ca3af);
    font-size: 0.75rem;
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
    color: var(--admin-color-text, #e5e7eb);
  }

  .rendition-size {
    font-size: 0.7rem;
    color: var(--admin-color-text-muted, #9ca3af);
  }
</style>
