<script lang="ts">
import {
  EntityAttributeList,
  EntityDetail,
  EntityDetailModule,
  MediaEditDialog,
  MediaReplaceFileForm,
  MediaPreviewTab,
  MediaRenditionsSection,
  MediaUsageList,
  MediaVersionActionDialogs,
  MediaVersionPreviewDialog,
  MediaVersionsList
} from "@decodelabs/underlay/templates";
import { MediaDetailWorkflowPage } from "@decodelabs/underlay/templates";
import {
  canActivateMediaVersion,
  canDeleteMediaVersion,
  canPreviewMedia,
  canPreviewMediaVersion,
  createClosedMediaEditDialogState,
  createMediaEditDialogDraft,
  createMediaVersionDialogStateController,
  getMediaVersionPreviewUrl,
  isCurrentMediaVersion,
  isImageMedia,
  isPdfMedia
} from "@decodelabs/underlay/runtime/media/detail";
import { formatFileSize } from "@decodelabs/underlay/runtime/media/upload";
import {
  getMediaKindLabel,
  getMediaVersionStateAccent,
  getMediaVisibilityLabel
} from "@decodelabs/underlay/runtime/media/types";
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
  PageLoading
} from "@inflatable-cookie/poodle-svelte";
import {
  Callout as PoodleCallout,
  Code,
  Dialog as PoodleDialog,
} from "@inflatable-cookie/poodle-svelte";
import type { PageData } from "./$types";
import { goto } from "$app/navigation";
import {
  Button as PoodleButton,
  IconButton as PoodleIconButton,
  TimeAgo
} from "@inflatable-cookie/poodle-svelte";
import {
  mediaCommands,
  type MediaDetail,
  type PagedListResponse,
  type MediaVersion,
  type MediaUsage,
  MediaKind,
  MediaVisibility,
  MediaVersionState
} from "@api-client";
import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
import { auth, authLoading, currentUser } from "$lib/stores/auth";
import { isPreconditionFailed } from "$lib/utils/api-errors";
import MediaActionsMenu from "$lib/components/MediaActionsMenu.svelte";
import { MAX_FILE_SIZE, replaceUpload } from "$lib/utils/upload-pipeline";
import { publicApiConfig } from "$lib/config/public-api";
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
      defaultValue: {
        data: [],
        total: 0,
        hasMore: false
      } as PagedListResponse<MediaVersion>
    }
  );

  const usagesData = useAuthenticatedData(
    async (fetchFn, token) => mediaCommands.listUsages(mediaId, fetchFn, token),
    {
      getAuthLoading: () => true,
      defaultValue: {
        data: [],
        total: 0,
        hasMore: false
      } as PagedListResponse<MediaUsage>
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

  const versions = $derived(versionsData.data?.data ?? []);
  const usages = $derived(usagesData.data?.data ?? []);
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
    const draft = createMediaEditDialogDraft(media);
    editTitle = draft.title;
    editFilename = draft.filename;
    editVisibility = draft.visibility;
    editDialogError = null;
    editDialogOpen = true;
  }

  function closeEditDialog() {
    const next = createClosedMediaEditDialogState();
    editDialogOpen = next.open;
    editDialogError = next.error;
    editDialogSubmitting = next.submitting;
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
  let replaceDialogOpen = $state(false);
  let previewDialogOpen = $state(false);
  let previewVersion = $state<MediaVersion | null>(null);
  const versionDialogs = createMediaVersionDialogStateController<MediaVersion>();

  $effect(() => {
    if (!previewDialogOpen) {
      previewVersion = null;
    }
  });
  async function handleReplaceSuccess() {
    replaceDialogOpen = false;
    reloadKey++;
    await versionsData.refetch();
  }

  async function confirmActivate() {
    if (!browser || !media || !versionDialogs.selectedVersion) return;

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await mediaCommands.activateVersion(
        media.id,
        versionDialogs.selectedVersion.id,
        window.fetch.bind(window),
        token
      );
      toastStore.push({ variant: "success", message: "Version activated" });
      versionDialogs.clear();
      reloadKey++;
      await Promise.all([versionsData.refetch(), usagesData.refetch()]);
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to activate version";
      toastStore.push({ variant: "error", message });
    }
  }

  async function confirmDelete() {
    if (!browser || !media || !versionDialogs.selectedVersion) return;

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await mediaCommands.deleteVersion(
        media.id,
        versionDialogs.selectedVersion.id,
        window.fetch.bind(window),
        token
      );
      toastStore.push({ variant: "success", message: "Version deleted" });
      versionDialogs.clear();
      reloadKey++;
      await Promise.all([versionsData.refetch(), usagesData.refetch()]);
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete version";
      toastStore.push({ variant: "error", message });
    }
  }

  function isCurrentVersion(version: MediaVersion): boolean {
    return isCurrentMediaVersion(media?.currentVersionId, version);
  }

  function canActivateVersion(version: MediaVersion): boolean {
    return canActivateMediaVersion(media?.currentVersionId, version);
  }

  function canDeleteVersion(version: MediaVersion): boolean {
    return canDeleteMediaVersion(media?.currentVersionId, version);
  }

  function canPreviewVersion(version: MediaVersion): boolean {
    return media ? canPreviewMediaVersion(media.kind, version) : false;
  }

  function openVersionPreview(version: MediaVersion) {
    if (!canPreviewVersion(version)) return;
    previewVersion = version;
    previewDialogOpen = true;
  }

  // Derived media URL
  const mediaPreviewUrl = $derived.by(() => {
    if (!media?.currentVersion) return null;
    return getMediaVersionPreviewUrl(media.currentVersion);
  });

  const showPreviewTab = $derived(
    media &&
    media.currentVersion?.state === MediaVersionState.Ready &&
    canPreviewMedia(media.kind, media.currentVersion?.mimeType ?? null) &&
    !!mediaPreviewUrl
  );

  const mediaTabs = $derived([
    {
      id: "details",
      label: "Details",
      content: detailsTabSnippet
    },
    ...(showPreviewTab
      ? [{
          id: "preview",
          label: "Preview",
          content: previewTabSnippet
        }]
      : []),
    {
      id: "usage",
      label: "Usage",
      count: usageCount,
      content: usageTabSnippet
    }
  ]);

</script>

<MediaDetailWorkflowPage
  item={media}
  backHref={backInfo.href}
  backLabel={backInfo.label}
  backIsContextual={backInfo.isContextual ?? false}
  dataLoader={mediaLoader}
  reloadKey={reloadKey}
  onTabChange={(tabId) => { activeTab = tabId; }}
  headerActions={headerActionsSnippet}
  tabs={mediaTabs}
  tabsVariant="underline"
/>

{#snippet headerActionsSnippet(_loadedMedia: MediaDetail)}
  {#if media}
    <MediaActionsMenu
      {media}
      trigger={mediaActionsTriggerSnippet}
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

{#snippet detailsTabSnippet(_loadedMedia: MediaDetail)}
  <EntityDetail>
    {#snippet children()}
      <EntityAttributeList
        title={null}
        columns={2}
        items={[
          {
            label: "Original Filename",
            value: originalFilenameSnippet,
            layout: "stacked",
            presentation: "surface"
          },
          ...(media?.currentVersion
            ? [
                {
                  label: "File Size",
                  value: fileSizeSnippet,
                  layout: "stacked" as const,
                  presentation: "surface" as const
                },
                {
                  label: "MIME Type",
                  value: mimeTypeSnippet,
                  layout: "stacked" as const,
                  presentation: "surface" as const
                }
              ]
            : []),
          {
            label: "Created",
            value: createdSnippet,
            layout: "stacked",
            presentation: "surface"
          },
          {
            label: "Last Updated",
            value: updatedSnippet,
            layout: "stacked",
            presentation: "surface"
          },
          ...(media?.deletedAt
            ? [{
                label: "Deleted",
                value: deletedAtSnippet,
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
              {#snippet actions()}
                <PoodleButton type="button" variant="ghost" size="sm" onclick={() => versionsData.refetch()}>
                  Retry
                </PoodleButton>
              {/snippet}
            </PoodleCallout>
          {/snippet}
        </EntityDetailModule>
      {:else}
        <EntityDetailModule>
          {#snippet children()}
            <MediaVersionsList
              {versions}
              onUploadNewVersion={() => { replaceDialogOpen = true; }}
              getVersionStateAccent={getMediaVersionStateAccent}
              {canPreviewVersion}
              onOpenVersionPreview={openVersionPreview}
              {formatFileSize}
              {isCurrentVersion}
              {canActivateVersion}
              {canDeleteVersion}
              onRequestActivate={versionDialogs.requestActivate}
              onRequestDelete={versionDialogs.requestDelete}
            />
          {/snippet}
        </EntityDetailModule>
      {/if}

      <EntityDetailModule span="full">
        {#snippet children()}
          <MediaRenditionsSection renditions={_loadedMedia.currentVersion?.renditions ?? []} {formatFileSize} />
        {/snippet}
      </EntityDetailModule>
    {/snippet}
  </EntityDetail>
{/snippet}

{#snippet previewTabSnippet(_loadedMedia: MediaDetail)}
  <EntityDetail>
    {#snippet children()}
      <EntityDetailModule span="full">
        {#snippet children()}
          <MediaPreviewTab
            media={media}
            mediaUrl={mediaPreviewUrl}
            isImage={isImageMedia}
            isPdf={isPdfMedia}
          />
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
              {#snippet actions()}
                <PoodleButton type="button" variant="ghost" size="sm" onclick={() => usagesData.refetch()}>
                  Retry
                </PoodleButton>
              {/snippet}
            </PoodleCallout>
          {/snippet}
        </EntityDetailModule>
      {:else}
        <EntityDetailModule span="full">
          {#snippet children()}
            <MediaUsageList {usages} />
          {/snippet}
        </EntityDetailModule>
      {/if}
    {/snippet}
  </EntityDetail>
{/snippet}

{#if media}

  <PoodleDialog
    bind:open={replaceDialogOpen}
    title="Replace file"
    description={`Upload a new version for ${media.title || media.originalFilename || media.id}.`}
    width="lg"
    showCloseButton
  >
    <MediaReplaceFileForm
      mediaId={media.id}
      maxFileSize={MAX_FILE_SIZE}
      replaceUpload={({ file, mediaId: replaceMediaId, onProgress }) => {
        const token = auth.getToken();
        if (!token) throw new Error("Not authenticated");
        return replaceUpload({
          file,
          mediaId: replaceMediaId,
          fetchFn: fetch,
          accessToken: token,
          onProgress
        });
      }}
      onToast={(variant, message) => {
        toastStore.push({ variant, message });
      }}
      onCancel={() => { replaceDialogOpen = false; }}
      onSuccess={handleReplaceSuccess}
    />
  </PoodleDialog>

  <MediaEditDialog
    bind:open={editDialogOpen}
    subtitle={`For ${media.title || media.originalFilename || media.id}`}
    error={editDialogError}
    submitting={editDialogSubmitting}
    bind:titleValue={editTitle}
    bind:filenameValue={editFilename}
    bind:visibilityValue={editVisibility}
    visibilityOptions={editVisibilityOptions}
    onCancel={closeEditDialog}
    onSubmit={handleEditSubmit}
  />

  <MediaVersionActionDialogs
    bind:activateDialogOpen={versionDialogs.activateDialogOpen}
    bind:deleteDialogOpen={versionDialogs.deleteDialogOpen}
    selectedVersion={versionDialogs.selectedVersion}
    onConfirmActivate={confirmActivate}
    onCancelActivate={versionDialogs.clear}
    onConfirmDelete={confirmDelete}
    onCancelDelete={versionDialogs.clear}
  />

  <MediaVersionPreviewDialog
    bind:open={previewDialogOpen}
    {previewVersion}
    mediaKind={media.kind}
    getPreviewUrl={getMediaVersionPreviewUrl}
    isImage={isImageMedia}
    isPdf={isPdfMedia}
  />
{/if}

<style>
</style>
