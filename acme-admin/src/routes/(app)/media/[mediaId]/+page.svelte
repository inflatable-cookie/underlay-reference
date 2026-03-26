<script lang="ts">
  import {
    AlertDialog as PoodleAlertDialog,
    Callout as PoodleCallout,
    Dialog as PoodleDialog
  } from "@poodle/svelte-primitives";
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";
  import { env } from "$env/dynamic/public";
  import {
    DetailPageShell,
    DetailMeta,
    DetailMetaId,
    DetailMetaSeparator,
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
    Code,
    DetailsCard,
    DetailsItem,
    DetailsSection,
    EmptyState,
        InlineListCard,
    InlineListItem,
    PageLoading,
    TimeAgo
  } from "@decodelabs/underlay/components";
  import {
    Button as PoodleButton,
    Field as PoodleField,
    FormActions as PoodleFormActions,
    IconButton as PoodleIconButton,
    Pill as PoodlePill,
    Select as PoodleSelect,
    TextInput as PoodleTextInput
  } from "@poodle/svelte-primitives";
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
  import { isPreconditionFailed } from "$lib/utils/api-errors";
  import { getMediaMetaAccent, getMediaMetaTone } from "$lib/utils/accents";
  import { uploadIcon } from "$lib/ui/poodle-icon-nodes";
  import MediaActionsMenu from "$lib/components/MediaActionsMenu.svelte";
  import Check from "lucide-svelte/icons/check";
  import Plus from "lucide-svelte/icons/plus";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import { browser } from "$app/environment";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  const toastStore = useToasts();
  const mediaId = $derived(data.mediaId);

  // Load media detail first; load heavy tab data lazily.
  const mediaData = useAuthenticatedData(
    async (fetchFn, token) => {
      const result = await mediaCommands.getMediaWithEtag(mediaId, fetchFn, token);
      return { media: result.data, etag: result.etag };
    },
    {}
  );

  const versionsData = useAuthenticatedData(
    async (fetchFn, token) => mediaCommands.listVersions(mediaId, fetchFn, token),
    {
      getAuthLoading: () => true,
      defaultValue: [] as MediaVersion[]
    }
  );

  const usagesData = useAuthenticatedData(
    async (fetchFn, token) => mediaCommands.listUsages(mediaId, fetchFn, token),
    {
      getAuthLoading: () => true,
      defaultValue: [] as MediaUsage[]
    }
  );

  let activeTab = $state("details");

  $effect(() => {
    if (activeTab === "details") {
      versionsData.tryFetch($authLoading, $currentUser);
    }
  });

  $effect(() => {
    if (activeTab === "usage") {
      usagesData.tryFetch($authLoading, $currentUser);
    }
  });

  const media = $derived(mediaData.data?.media);
  let currentEtag = $state<string | null>(null);

  $effect(() => {
    if (mediaData.data?.etag) {
      currentEtag = mediaData.data.etag;
    }
  });
  const versions = $derived(versionsData.data ?? []);
  const usages = $derived(usagesData.data ?? []);
  const usageCount = $derived(media?.usageCount ?? 0);

  const backInfo = getBackButtonInfo("Back to media", "/media");
  const editVisibilityOptions = [
    { value: MediaVisibility.Public, label: "Public - accessible without login" },
    { value: MediaVisibility.Restricted, label: "Restricted - requires authentication" }
  ];

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

  function getVersionStateTone(state: MediaVersionState): "neutral" | "success" | "danger" {
    if (state === MediaVersionState.Ready) return "success";
    if (state === MediaVersionState.Failed) return "danger";
    return "neutral";
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
      const result = await mediaCommands.updateMediaWithEtag(
        media.id,
        {
          title: editTitle || null,
          originalFilename: editFilename || null,
          visibility: editVisibility as MediaVisibility
        },
        window.fetch.bind(window),
        token,
        { ifMatch: currentEtag ?? undefined }
      );
      currentEtag = result.etag;
      toastStore.push({ variant: "success", message: "Media updated" });
      closeEditDialog();
      await mediaData.refetch();
    } catch (e) {
      if (isPreconditionFailed(e)) {
        const latest = await mediaCommands.getMediaWithEtag(
          media.id,
          window.fetch.bind(window),
          token
        );
        currentEtag = latest.etag;
        editTitle = latest.data.title || "";
        editFilename = latest.data.originalFilename || "";
        editVisibility = latest.data.visibility;
        await mediaData.refetch();
        editDialogError = "This media item was changed in another session. Review the latest values, reapply your edits, and save again.";
        return;
      }
      const message = e instanceof Error ? e.message : "Failed to update media";
      editDialogError = message;
    } finally {
      editDialogSubmitting = false;
    }
  }

  // Version actions state
  let activateDialogOpen = $state(false);
  let deleteDialogOpen = $state(false);
  let previewDialogOpen = $state(false);
  let previewVersion = $state<MediaVersion | null>(null);

  $effect(() => {
    if (!previewDialogOpen) {
      previewVersion = null;
    }
  });
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
      await Promise.all([mediaData.refetch(), versionsData.refetch()]);
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
      await Promise.all([mediaData.refetch(), versionsData.refetch()]);
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

  function getPreviewUrl(version: MediaVersion): string | null {
    if (version.url) return version.url;
    const imageRendition = version.renditions?.find((rendition) => rendition.url && rendition.mimeType?.startsWith("image/"));
    return imageRendition?.url ?? null;
  }

  function canPreviewVersion(version: MediaVersion): boolean {
    if (!media) return false;
    if (version.state !== MediaVersionState.Ready) return false;
    if (media.kind === MediaKind.Image) return Boolean(getPreviewUrl(version));
    if (media.kind === MediaKind.Pdf) return Boolean(version.url);
    return false;
  }

  function openVersionPreview(version: MediaVersion) {
    if (!canPreviewVersion(version)) return;
    previewVersion = version;
    previewDialogOpen = true;
  }

  // Derived media URL
  const mediaPreviewUrl = $derived.by(() => {
    if (!media?.currentVersion) return null;
    return getPreviewUrl(media.currentVersion);
  });

  const showPreviewTab = $derived(
    media &&
    media.currentVersion?.state === MediaVersionState.Ready &&
    canPreview(media.kind, media.currentVersion?.mimeType ?? null) &&
    !!mediaPreviewUrl
  );

  const mediaTabs = $derived([
    { value: "details", label: "Details" },
    ...(showPreviewTab ? [{ value: "preview", label: "Preview" }] : []),
    { value: "usage", label: "Usage", count: usageCount }
  ]);
</script>

{#if mediaData.loading}
  <PageLoading message="Loading media..." />
{:else if mediaData.error}
  <PoodleCallout tone="danger" message={mediaData.error} announceMode="polite" />
{:else if media}
  <DetailPageShell
    section="Media"
    title={media.title || media.originalFilename || "Untitled"}
    backHref={backInfo.href}
    backLabel={backInfo.label}
    backIsContextual={backInfo.isContextual ?? false}
    bannerMessage={media.deletedAt ? "This media has been soft-deleted." : undefined}
    tabs={mediaTabs}
    bind:activeTab
    tabsHistoryKey="tab"
  >
    {#snippet meta()}
      <DetailMeta>
        <DetailMetaId value={media.id} />
        <DetailMetaSeparator />
        <PoodlePill tone="neutral" appearance="badge" size="lg">{getMediaKindLabel(media.kind)}</PoodlePill>
        <PoodlePill tone="neutral" appearance="badge" size="lg">
          {getMediaVisibilityLabel(media.visibility)}
        </PoodlePill>
        {#if media.deletedAt}
          <PoodlePill tone={getMediaMetaTone("deleted")} appearance="badge" size="lg">Deleted</PoodlePill>
        {/if}
      </DetailMeta>
    {/snippet}

    {#snippet actions()}
      <MediaActionsMenu
        {media}
        onEditRequest={openEditDialog}
        onSoftDeleteSuccess={() => goto("/media")}
        onRestoreSuccess={() => mediaData.refetch()}
      />
    {/snippet}

    {#snippet tabContent(tab)}
      {#if tab === "details"}
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
              <PoodleIconButton
                type="button"
                variant="primary"
                size="sm"
                icon={uploadIcon}
                ariaLabel="Upload new version"
                on:click={() => goto(`/media/upload?replace=${media.id}`)}
              />
            {/snippet}
            {#if activeTab === "details" && versionsData.loading}
              <PageLoading message="Loading versions..." />
            {:else if versionsData.error}
              <PoodleCallout tone="danger" message={versionsData.error} announceMode="polite" />
            {:else}
              {#each versions as version (version.id)}
                <InlineListItem
                  label={version.sha256 ?? "No hash"}
                  accent={getMediaVersionStateAccent(version.state)}
                  onclick={canPreviewVersion(version) ? () => openVersionPreview(version) : undefined}
                >
                  {#snippet sublabelContent()}
                    {formatFileSize(version.byteSize)} · <Code>{version.mimeType ?? "Unknown type"}</Code> · <TimeAgo date={version.createdAt} short />
                  {/snippet}
                  {#snippet trailing()}
                    <PoodlePill tone={getVersionStateTone(version.state)} appearance="badge" size="lg">
                      {getMediaVersionStateLabel(version.state)}
                    </PoodlePill>
                    {#if isCurrentVersion(version)}
                      <PoodlePill tone={getMediaMetaTone("current")} appearance="badge" size="lg">Current</PoodlePill>
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
            {/if}
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
      {:else if tab === "preview"}
        <div class="media-preview-container">
          {#if mediaPreviewUrl}
            {#if isImage(media.kind)}
              <img
                src={mediaPreviewUrl}
                alt={media.title || media.originalFilename || "Media preview"}
                class="media-preview-image"
              />
            {:else if isPdf(media.kind)}
              <iframe
                src={mediaPreviewUrl}
                title={media.title || media.originalFilename || "PDF preview"}
                class="media-preview-pdf"
              ></iframe>
            {/if}
          {:else}
            <EmptyState title="Preview not available" description="Preview is not available for this version." variant="compact" />
          {/if}
        </div>
      {:else if tab === "usage"}
        <div class="underlay-details-content">
          {#if activeTab === "usage" && usagesData.loading}
            <PageLoading message="Loading usage..." />
          {:else if usagesData.error}
            <PoodleCallout tone="danger" message={usagesData.error} announceMode="polite" />
          {:else if usages.length === 0}
            <EmptyState title="No usage found" description="This media is not used anywhere yet." variant="compact" />
          {:else}
            <InlineListCard
              title="Usages"
              hasItems={true}
            >
              {#each usages as usage}
                <InlineListItem
                  label={usage.usedByType}
                  accent={getMediaMetaAccent("usage")}
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
      {/if}
    {/snippet}
  </DetailPageShell>

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
          <PoodleField id="edit-title" label="Title" let:describedBy>
            <PoodleTextInput
              id="edit-title"
              name="title"
              value={editTitle}
              describedBy={describedBy}
              placeholder="Enter a title for this media"
              disabled={submitting}
              on:valueChange={(event) => { editTitle = event.detail.value; }}
            />
          </PoodleField>

          <PoodleField id="edit-filename" label="Filename" hint="The filename shown when downloading" let:describedBy>
            <PoodleTextInput
              id="edit-filename"
              name="filename"
              value={editFilename}
              describedBy={describedBy}
              placeholder="e.g. document.pdf"
              disabled={submitting}
              on:valueChange={(event) => { editFilename = event.detail.value; }}
            />
          </PoodleField>

          <PoodleField id="edit-visibility" label="Visibility" let:describedBy>
            <PoodleSelect
              id="edit-visibility"
              name="visibility"
              value={editVisibility}
              describedBy={describedBy}
              options={editVisibilityOptions}
              disabled={submitting}
              on:valueChange={(event) => { editVisibility = event.detail.value; }}
            />
          </PoodleField>
        </div>

        <PoodleFormActions align="between">
          <PoodleButton type="button" variant="ghost" disabled={submitting} on:click={closeEditDialog}>
            Cancel
          </PoodleButton>
          <PoodleButton type="submit" variant="primary" disabled={submitting}>
            {submitting ? "Saving..." : "Save"}
          </PoodleButton>
        </PoodleFormActions>
      </form>
    {/snippet}
  </FormDialog>

  <!-- Activate Version Dialog -->
  <PoodleAlertDialog
    bind:open={activateDialogOpen}
    title="Activate version?"
    description="This will set this version as the current active version for this media item."
    confirmLabel="Activate"
    cancelLabel="Cancel"
    onConfirm={confirmActivate}
    onCancel={cancelActivate}
    tone="warning"
  >
    {#if selectedVersion}
      <p>
        Version: <Code>{selectedVersion.sha256?.slice(0, 16) ?? selectedVersion.id}...</Code>
      </p>
    {/if}
  </PoodleAlertDialog>

  <!-- Delete Version Dialog -->
  <PoodleAlertDialog
    bind:open={deleteDialogOpen}
    title="Delete version?"
    description="This will permanently delete this version and its stored file. This action cannot be undone."
    confirmLabel="Delete"
    cancelLabel="Cancel"
    onConfirm={confirmDelete}
    onCancel={cancelDelete}
    tone="danger"
  >
    {#if selectedVersion}
      <p>
        Version: <Code>{selectedVersion.sha256?.slice(0, 16) ?? selectedVersion.id}...</Code>
      </p>
    {/if}
  </PoodleAlertDialog>

  <PoodleDialog
    bind:open={previewDialogOpen}
    title="Version preview"
    contentClassName="version-preview-dialog"
    showCloseButton
  >
    {#if previewVersion}
      {@const previewUrl = getPreviewUrl(previewVersion)}
      {#if previewUrl}
        {#if isImage(media.kind)}
          <img class="version-preview-image" src={previewUrl} alt="Version preview" />
        {:else if isPdf(media.kind)}
          <iframe class="version-preview-frame" title="Version preview" src={previewUrl}></iframe>
        {:else}
          <p>Preview not available for this file type.</p>
        {/if}
      {:else}
        <p>Preview not available for this version.</p>
      {/if}
    {/if}
  </PoodleDialog>
{/if}

<style>
  .deleted-date {
    color: var(--color-danger, #ef4444);
  }

  .usage-field {
    color: var(--admin-color-text-muted);
  }

  :global(.version-preview-dialog) {
    width: min(900px, 90vw);
  }

  .version-preview-image {
    display: block;
    max-width: 100%;
    max-height: 70vh;
    margin: 0 auto;
    border-radius: 0.5rem;
  }

  .version-preview-frame {
    width: 100%;
    height: 70vh;
    border: 0;
    border-radius: 0.5rem;
    background: var(--admin-color-surface-subtle);
  }

  .form-fields {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-bottom: 1.5rem;
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
