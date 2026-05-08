<script lang="ts">
import {
  EntityAttributeList,
  EntityDetail,
  EntityDetailModule,
  EntityDetailPage,
  EntityInlineListModule
} from "@decodelabs/underlay/templates";
import {
  getMediaKindLabel,
  getMediaVisibilityLabel,
  getMediaVersionStateLabel,
  getMediaVersionStateAccent
} from "@decodelabs/underlay/runtime/media";
import {
  useToasts,
} from "@decodelabs/underlay/runtime/feedback";
import {
  getBackButtonInfo,
} from "@decodelabs/underlay/runtime/navigation";
import {
  useAuthenticatedData,
} from "@decodelabs/underlay/runtime/auth";
import {
  EmptyState as PoodleEmptyState,
  FormDialog,
  PageLoading
} from "@poodle/svelte";
import {
  AlertDialog as PoodleAlertDialog,
  Callout as PoodleCallout,
  Code,
  Dialog as PoodleDialog,
  formatFileSize
} from "@poodle/svelte";
import type { PageData } from "./$types";
import { goto } from "$app/navigation";
import { env } from "$env/dynamic/public";
import {
  Button as PoodleButton,
  Field as PoodleField,
  FormActions as PoodleFormActions,
  IconButton as PoodleIconButton,
  Pill as PoodlePill,
  Select as PoodleSelect,
  TextInput as PoodleTextInput,
  TimeAgo
} from "@poodle/svelte";
import {
  mediaCommands,
  type MediaDetail,
  type MediaVersion,
  type MediaUsage,
  MediaKind,
  MediaVisibility,
  MediaVersionState
} from "@api-client";
import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
import { auth, authLoading, currentUser } from "$lib/stores/auth";
import { isPreconditionFailed } from "$lib/utils/api-errors";
import { getMediaMetaAccent, getMediaMetaTone } from "$lib/utils/accents";
import { uploadIcon } from "$lib/ui/poodle-icon-nodes";
import MediaActionsMenu from "$lib/components/MediaActionsMenu.svelte";
import Check from "lucide-svelte/icons/check";
import Trash2 from "lucide-svelte/icons/trash-2";
import { browser } from "$app/environment";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  const toastStore = useToasts();
  const mediaId = $derived(data.mediaId);

  let media = $state<MediaDetail | null>(null);
  let currentEtag = $state<string | null>(null);
  let reloadKey = $state(0);

  async function mediaLoader(fetchFn: typeof window.fetch, token: string | null) {
    if (!token) throw new Error("Not authenticated");
    const result = await mediaCommands.getMediaWithEtag(mediaId, fetchFn, token);
    media = result.data;
    currentEtag = result.etag;
    return result.data;
  }

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
      media = result.data;
      toastStore.push({ variant: "success", message: "Media updated" });
      closeEditDialog();
      reloadKey++;
    } catch (e) {
      if (isPreconditionFailed(e)) {
        const latest = await mediaCommands.getMediaWithEtag(
          media.id,
          window.fetch.bind(window),
          token
        );
        currentEtag = latest.etag;
        media = latest.data;
        editTitle = latest.data.title || "";
        editFilename = latest.data.originalFilename || "";
        editVisibility = latest.data.visibility;
        reloadKey++;
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
      reloadKey++;
      await Promise.all([versionsData.refetch(), usagesData.refetch()]);
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
      reloadKey++;
      await Promise.all([versionsData.refetch(), usagesData.refetch()]);
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
    const imageRendition = version.renditions?.find(
      (rendition: MediaVersion["renditions"][number]) =>
        rendition.url && rendition.mimeType?.startsWith("image/")
    );
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
    {
      id: "details",
      label: "Details",
      content: detailsTabSnippet as never
    },
    ...(showPreviewTab
      ? [{
          id: "preview",
          label: "Preview",
          content: previewTabSnippet as never
        }]
      : []),
    {
      id: "usage",
      label: "Usage",
      count: usageCount,
      content: usageTabSnippet as never
    }
  ]);

  const headerMeta = $derived.by(() => {
    const items = [
      { label: "ID", value: idSnippet as never },
      { label: "", value: kindSnippet as never, separator: false },
      { label: "", value: visibilitySnippet as never, separator: false }
    ];

    if (media?.deletedAt) {
      items.push({ label: "", value: deletedSnippet as never, separator: false });
    }

    return items;
  });
</script>

<EntityDetailPage
  title={media?.title || media?.originalFilename || "Untitled"}
  section="Media"
  backHref={backInfo.href}
  backLabel={backInfo.label}
  bannerMessage={media?.deletedAt ? "This media has been soft-deleted." : undefined}
  dataLoader={mediaLoader}
  reloadKey={reloadKey}
  onTabChange={(tabId) => { activeTab = tabId; }}
  meta={headerMeta as never}
  headerActions={headerActionsSnippet as never}
  tabs={mediaTabs as never}
/>

{#snippet headerActionsSnippet(_loadedMedia: MediaDetail)}
  {#if media}
    <MediaActionsMenu
      {media}
      trigger={mediaActionsTriggerSnippet as never}
      onEditRequest={openEditDialog}
      onSoftDeleteSuccess={() => goto("/media")}
      onRestoreSuccess={() => { reloadKey++; }}
    />
  {/if}
{/snippet}

{#snippet mediaActionsTriggerSnippet()}
  <PoodleIconButton
    type="button"
    icon="ellipsis"
    variant="secondary"
    ariaLabel="Media actions"
    tooltip="Actions"
  />
{/snippet}

{#snippet idSnippet()}
  {#if media}
    <Code inline inlineVariant="plain" typography="inline" source={media.id} showCopyButton />
  {/if}
{/snippet}

{#snippet kindSnippet()}
  {#if media}
    <PoodlePill tone="neutral" appearance="badge" size="sm" typography="inherit">
      {getMediaKindLabel(media.kind)}
    </PoodlePill>
  {/if}
{/snippet}

{#snippet visibilitySnippet()}
  {#if media}
    <PoodlePill tone="neutral" appearance="badge" size="sm" typography="inherit">
      {getMediaVisibilityLabel(media.visibility)}
    </PoodlePill>
  {/if}
{/snippet}

{#snippet deletedSnippet()}
  <PoodlePill tone={getMediaMetaTone("deleted")} appearance="badge" size="sm" typography="inherit">
    Deleted
  </PoodlePill>
{/snippet}

{#snippet createdSnippet()}
  {#if media}
    <TimeAgo datetime={media.createdAt} tooltipFormat="datetime" />
  {/if}
{/snippet}

{#snippet updatedSnippet()}
  {#if media}
    <TimeAgo datetime={media.updatedAt} tooltipFormat="datetime" />
  {/if}
{/snippet}

{#snippet deletedAtSnippet()}
  {#if media?.deletedAt}
    <TimeAgo datetime={media.deletedAt} tooltipFormat="datetime" />
  {/if}
{/snippet}

{#snippet mimeTypeSnippet()}
  {#if media?.currentVersion}
    <Code inline source={media.currentVersion.mimeType ?? "—"} />
  {/if}
{/snippet}

{#snippet originalFilenameSnippet()}
  {media?.originalFilename?.trim() || "—"}
{/snippet}

{#snippet fileSizeSnippet()}
  {#if media?.currentVersion?.byteSize != null}
    {formatFileSize(media.currentVersion.byteSize)}
  {:else}
    —
  {/if}
{/snippet}

{#snippet versionsActionsSnippet()}
  {#if media}
    {@const replaceHref = `/media/upload?replace=${media.id}`}
    <PoodleIconButton
      type="button"
      variant="primary"
      size="sm"
      icon={uploadIcon}
      ariaLabel="Upload new version"
      on:click={() => goto(replaceHref)}
    />
  {/if}
{/snippet}

{#snippet versionItemSnippet(version: MediaVersion)}
  {#if canPreviewVersion(version)}
    <button
      type="button"
      class="inline-list-card__item-content inline-list-card__item-content--button"
      onclick={() => openVersionPreview(version)}
    >
      <span class="inline-list-card__dot" style:--inline-list-accent={getMediaVersionStateAccent(version.state)}></span>
      <span class="inline-list-card__label-group">
        <span class="inline-list-card__label">{version.sha256 ?? "No hash"}</span>
        <span class="inline-list-card__sublabel">
          {formatFileSize(version.byteSize)} · <Code inline source={version.mimeType ?? "Unknown type"} /> · <TimeAgo datetime={version.createdAt} short />
        </span>
      </span>
    </button>
  {:else}
    <div class="inline-list-card__item-content">
      <span class="inline-list-card__dot" style:--inline-list-accent={getMediaVersionStateAccent(version.state)}></span>
      <span class="inline-list-card__label-group">
        <span class="inline-list-card__label">{version.sha256 ?? "No hash"}</span>
        <span class="inline-list-card__sublabel">
          {formatFileSize(version.byteSize)} · <Code inline source={version.mimeType ?? "Unknown type"} /> · <TimeAgo datetime={version.createdAt} short />
        </span>
      </span>
    </div>
  {/if}

  <div class="inline-list-card__trailing">
    <PoodlePill tone={getVersionStateTone(version.state)} appearance="badge" size="sm">
      {getMediaVersionStateLabel(version.state)}
    </PoodlePill>
    {#if isCurrentVersion(version)}
      <PoodlePill tone={getMediaMetaTone("current")} appearance="badge" size="sm">
        Current
      </PoodlePill>
    {/if}
  </div>

  <div class="inline-list-card__actions">
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
  </div>
{/snippet}

{#snippet usageItemSnippet(usage: MediaUsage)}
  <div class="inline-list-card__item-content inline-list-card__item-content--usage">
    <span class="inline-list-card__dot" style:--inline-list-accent={getMediaMetaAccent("usage")}></span>
    <span class="inline-list-card__label-group">
      <span class="inline-list-card__label">{usage.usedByType}</span>
      <span class="inline-list-card__sublabel">
        <Code inline source={usage.usedById} />
        {#if usage.field}
          <span class="usage-field"> · {usage.field}</span>
        {/if}
      </span>
    </span>
  </div>
{/snippet}

{#snippet detailsTabSnippet(_loadedMedia: MediaDetail)}
  <EntityDetail>
    {#snippet children()}
      <EntityAttributeList
        title={null}
        columns={2}
        items={[
          {
            label: "Original Filename",
            value: originalFilenameSnippet as never,
            layout: "stacked",
            presentation: "surface"
          },
          ...(media?.currentVersion
            ? [
                {
                  label: "File Size",
                  value: fileSizeSnippet as never,
                  layout: "stacked" as const,
                  presentation: "surface" as const
                },
                {
                  label: "MIME Type",
                  value: mimeTypeSnippet as never,
                  layout: "stacked" as const,
                  presentation: "surface" as const
                }
              ]
            : []),
          {
            label: "Created",
            value: createdSnippet as never,
            layout: "stacked",
            presentation: "surface"
          },
          {
            label: "Last Updated",
            value: updatedSnippet as never,
            layout: "stacked",
            presentation: "surface"
          },
          ...(media?.deletedAt
            ? [{
                label: "Deleted",
                value: deletedAtSnippet as never,
                layout: "stacked" as const,
                presentation: "surface" as const
              }]
            : [])
        ]}
      />

      {#if activeTab === "details" && versionsData.loading}
        <EntityDetailModule>
          {#snippet children()}
            <PageLoading presentation="inline" message="Loading versions..." />
          {/snippet}
        </EntityDetailModule>
      {:else if versionsData.error}
        <EntityDetailModule>
          {#snippet children()}
            <PoodleCallout tone="danger" title="Unable to load versions" message={versionsData.error} announceMode="polite">
              <svelte:fragment slot="actions">
                <PoodleButton type="button" variant="ghost" size="sm" onclick={() => versionsData.refetch()}>
                  Retry
                </PoodleButton>
              </svelte:fragment>
            </PoodleCallout>
          {/snippet}
        </EntityDetailModule>
      {:else}
        <EntityInlineListModule
          title="Versions"
          items={versions}
          item={versionItemSnippet as never}
          actions={versionsActionsSnippet as never}
          emptyMessage="No versions uploaded yet."
        />
      {/if}

      {#if _loadedMedia.currentVersion?.renditions && _loadedMedia.currentVersion.renditions.length > 0}
        <EntityDetailModule span="full">
          {#snippet children()}
            {@const renditions = _loadedMedia.currentVersion?.renditions ?? []}
            <div class="renditions-section">
              <h3>Renditions</h3>
              <div class="renditions-grid">
                {#each renditions as rendition}
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
            </div>
          {/snippet}
        </EntityDetailModule>
      {/if}
    {/snippet}
  </EntityDetail>
{/snippet}

{#snippet previewTabSnippet(_loadedMedia: MediaDetail)}
  <EntityDetail>
    {#snippet children()}
      <EntityDetailModule span="full">
        {#snippet children()}
          <div class="media-preview-container">
            {#if mediaPreviewUrl && media}
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
              <PoodleEmptyState title="Preview not available" message="Preview is not available for this version." size="compact" />
            {/if}
          </div>
        {/snippet}
      </EntityDetailModule>
    {/snippet}
  </EntityDetail>
{/snippet}

{#snippet usageTabSnippet(_loadedMedia: MediaDetail)}
  <EntityDetail>
    {#snippet children()}
      {#if activeTab === "usage" && usagesData.loading}
        <EntityDetailModule span="full">
          {#snippet children()}
            <PageLoading presentation="inline" message="Loading usage..." />
          {/snippet}
        </EntityDetailModule>
      {:else if usagesData.error}
        <EntityDetailModule span="full">
          {#snippet children()}
            <PoodleCallout tone="danger" title="Unable to load usage" message={usagesData.error} announceMode="polite">
              <svelte:fragment slot="actions">
                <PoodleButton type="button" variant="ghost" size="sm" onclick={() => usagesData.refetch()}>
                  Retry
                </PoodleButton>
              </svelte:fragment>
            </PoodleCallout>
          {/snippet}
        </EntityDetailModule>
      {:else if usages.length === 0}
        <EntityDetailModule span="full">
          {#snippet children()}
            <PoodleEmptyState title="No usage found" message="This media is not used anywhere yet." size="compact" />
          {/snippet}
        </EntityDetailModule>
      {:else}
        <EntityInlineListModule
          title="Usages"
          items={usages}
          item={usageItemSnippet as never}
          span="full"
        />
      {/if}
    {/snippet}
  </EntityDetail>
{/snippet}

{#if media}

  <!-- Edit Dialog -->
  <FormDialog
    bind:open={editDialogOpen}
    title="Edit Media"
    subtitle={`For ${media.title || media.originalFilename || media.id}`}
    error={editDialogError}
    submitting={editDialogSubmitting}
    showDefaultActions={false}
    on:cancel={closeEditDialog}
  >
    <form id="media-edit-form" onsubmit={handleEditSubmit}>
      <div class="form-fields">
          <PoodleField id="edit-title" label="Title" let:describedBy>
            <PoodleTextInput
              id="edit-title"
              name="title"
              value={editTitle}
              describedBy={describedBy}
              placeholder="Enter a title for this media"
              disabled={editDialogSubmitting}
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
              disabled={editDialogSubmitting}
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
              disabled={editDialogSubmitting}
              on:valueChange={(event) => { editVisibility = event.detail.value; }}
            />
          </PoodleField>
      </div>
    </form>

    <svelte:fragment slot="actions">
      <PoodleFormActions align="between">
        <PoodleButton type="button" variant="ghost" disabled={editDialogSubmitting} on:click={closeEditDialog}>
            Cancel
        </PoodleButton>
        <PoodleButton type="submit" form="media-edit-form" variant="primary" disabled={editDialogSubmitting}>
          {editDialogSubmitting ? "Saving..." : "Save"}
        </PoodleButton>
      </PoodleFormActions>
    </svelte:fragment>
  </FormDialog>

  <!-- Activate Version Dialog -->
  <PoodleAlertDialog
    bind:open={activateDialogOpen}
    title="Activate version?"
    description="This will set this version as the current active version for this media item."
    itemLabel="Version"
    itemValue={selectedVersion ? `${selectedVersion.sha256?.slice(0, 16) ?? selectedVersion.id}...` : null}
    confirmLabel="Activate version"
    cancelLabel="Cancel"
    onConfirm={confirmActivate}
    onCancel={cancelActivate}
    tone="warning"
  />

  <!-- Delete Version Dialog -->
  <PoodleAlertDialog
    bind:open={deleteDialogOpen}
    title="Permanently delete version?"
    description="This will permanently delete this version and its stored file. This cannot be undone."
    itemLabel="Version"
    itemValue={selectedVersion ? `${selectedVersion.sha256?.slice(0, 16) ?? selectedVersion.id}...` : null}
    confirmLabel="Permanently delete version"
    cancelLabel="Cancel"
    onConfirm={confirmDelete}
    onCancel={cancelDelete}
    tone="danger"
  />

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
  .usage-field {
    color: var(--poodle-color-text-secondary);
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
    background: var(--poodle-color-background-surface);
  }

  .form-fields {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .inline-list-card__trailing,
  .inline-list-card__actions {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }

  .inline-list-card__item-content {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 0.625rem;
    color: inherit;
    text-decoration: none;
  }

  .inline-list-card__item-content--usage {
    align-items: flex-start;
  }

  .inline-list-card__item-content--button {
    width: 100%;
    padding: 0;
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
  }

  .inline-list-card__dot {
    --inline-list-accent: var(--poodle-color-accent-base);
    width: 0.375rem;
    height: 0.375rem;
    border-radius: 999rem;
    background: var(--inline-list-accent);
    flex-shrink: 0;
  }

  .inline-list-card__label-group {
    min-width: 0;
    display: grid;
    gap: 0.125rem;
  }

  .inline-list-card__label {
    display: block;
    min-width: 0;
    color: var(--poodle-color-text-primary);
    font-size: 0.875rem;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .inline-list-card__sublabel {
    font-size: 0.75rem;
    color: var(--poodle-color-text-secondary);
  }

  .inline-list-card__actions button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.75rem;
    height: 1.75rem;
    padding: 0;
    border: 0;
    border-radius: var(--poodle-radius-control);
    background: transparent;
    color: var(--poodle-color-text-secondary);
    cursor: pointer;
    transition:
      background 120ms ease,
      color 120ms ease;
  }

  .inline-list-card__actions button:hover:not(:disabled) {
    background: color-mix(in srgb, var(--poodle-color-accent-base) 14%, transparent);
    color: var(--poodle-color-text-primary);
  }

  .inline-list-card__actions button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .inline-list-card__actions button:focus-visible {
    outline: var(--poodle-border-width-focus) solid var(--poodle-color-accent-focusRing);
    outline-offset: 0.125rem;
  }

  /* Full preview tab */
  .media-preview-container {
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding: 1rem;
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
    display: grid;
    gap: 1rem;
  }

  .renditions-section h3 {
    margin: 0;
    color: var(--poodle-color-text-secondary);
    font-family: var(--poodle-typography-label-family);
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .renditions-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 1rem;
  }

  .rendition-card {
    background: color-mix(in srgb, var(--poodle-surface) 93%, var(--poodle-color-text-primary));
    border: 0.0625rem solid color-mix(in srgb, var(--poodle-color-border-subtle) 74%, transparent);
    border-radius: calc(var(--poodle-radius-surface) - 0.1875rem);
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
    color: var(--poodle-color-text-secondary);
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
    color: var(--poodle-color-text-primary);
  }

  .rendition-size {
    font-size: 0.7rem;
    color: var(--poodle-color-text-secondary);
  }
</style>
