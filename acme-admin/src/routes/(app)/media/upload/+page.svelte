<script lang="ts">
import {
  validateFileType,
  ALLOWED_MEDIA_TYPES,
  REJECTED_VIDEO_TYPES,
  type UploadProgress,
} from "@decodelabs/underlay/runtime/media";
import {
  useToasts,
} from "@decodelabs/underlay/runtime/feedback";
import {
  PageHeader as PoodlePageHeader,
  PageLoading } from "@poodle/svelte";
  import { Callout as PoodleCallout,
  FileUpload,
  Progress,
  formatFileSize,
  type FileUploadItem } from "@poodle/svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { mediaCommands } from "@api-client";
  import { auth,
  authLoading,
  currentUser } from "$lib/stores/auth";
    import { Button as PoodleButton } from "@poodle/svelte";
  import Upload from "lucide-svelte/icons/upload";
  import AlertCircle from "lucide-svelte/icons/alert-circle";
  import CheckCircle from "lucide-svelte/icons/check-circle";
  import XCircle from "lucide-svelte/icons/x-circle";
  import X from "lucide-svelte/icons/x";
  import FileIcon from "lucide-svelte/icons/file";
  import {
    MAX_FILE_SIZE,
    checkDuplicate,
    createAndUpload,
    replaceUpload,
  } from "$lib/utils/upload-pipeline";

  const toastStore = useToasts();

  // Check if we're replacing an existing media (single file mode)
  const replaceMediaId = $derived($page.url.searchParams.get("replace"));
  const isBulkMode = $derived(!replaceMediaId);

  // Upload queue item type
  type UploadStatus = "pending" | "hashing" | "checking" | "creating" | "uploading" | "finalizing" | "done" | "error" | "duplicate";

  interface QueueItem {
    id: string;
    file: File;
    status: UploadStatus;
    progress: number;
    error?: string;
    title: string;
    hash?: string;
    mediaId?: string;
    duplicateOf?: { id: string; title: string };
  }

  // State
  let files = $state<FileUploadItem[]>([]);
  let uploadQueue = $state<QueueItem[]>([]);
  let uploading = $state(false);
  let error = $state<string | null>(null);

  // Single-file mode state (for replace)
  let singleTitle = $state("");
  let singleAltText = $state("");

  // Derived states
  const hasFiles = $derived(files.length > 0 || uploadQueue.length > 0);
  const pendingCount = $derived(uploadQueue.filter(q => q.status === "pending").length);
  const completedCount = $derived(uploadQueue.filter(q => q.status === "done").length);
  const errorCount = $derived(uploadQueue.filter(q => q.status === "error").length);
  const allDone = $derived(uploadQueue.length > 0 && uploadQueue.every(q => q.status === "done" || q.status === "error" || q.status === "duplicate"));

  // Add files to queue when selected (bulk mode only)
  $effect(() => {
    if (isBulkMode && files.length > 0) {
      addFilesToQueue();
    }
  });

  function addFilesToQueue() {
    const validationErrors: string[] = [];

    for (const fileItem of files) {
      const file = fileItem.file;

      // Check for video types (explicitly rejected)
      if (REJECTED_VIDEO_TYPES.includes(file.type as typeof REJECTED_VIDEO_TYPES[number])) {
        validationErrors.push(`${file.name}: Video uploads are not supported`);
        continue;
      }

      // Check file type
      if (!validateFileType(file, ALLOWED_MEDIA_TYPES)) {
        validationErrors.push(`${file.name}: Unsupported file type`);
        continue;
      }

      // Check file size
      if (file.size > MAX_FILE_SIZE) {
        validationErrors.push(`${file.name}: File too large (${formatFileSize(file.size)})`);
        continue;
      }

      // Check if already in queue
      const alreadyQueued = uploadQueue.some(
        q => q.file.name === file.name && q.file.size === file.size
      );
      if (alreadyQueued) {
        continue;
      }

      // Add to queue
      const title = file.name.replace(/\.[^/.]+$/, "").replace(/[-_]/g, " ");
      uploadQueue.push({
        id: crypto.randomUUID(),
        file,
        status: "pending",
        progress: 0,
        title
      });
    }

    // Clear file input
    files = [];

    if (validationErrors.length > 0) {
      error = validationErrors.join("\n");
    } else {
      error = null;
    }
  }

  function removeFromQueue(itemId: string) {
    uploadQueue = uploadQueue.filter(q => q.id !== itemId);
  }

  function getStatusLabel(status: UploadStatus): string {
    switch (status) {
      case "pending": return "Waiting";
      case "hashing": return "Hashing...";
      case "checking": return "Checking...";
      case "creating": return "Creating...";
      case "uploading": return "Uploading";
      case "finalizing": return "Finalizing...";
      case "done": return "Complete";
      case "error": return "Failed";
      case "duplicate": return "Duplicate";
      default: return "";
    }
  }

  async function handleUpload() {
    if (uploadQueue.length === 0) {
      error = "Please select files to upload";
      return;
    }

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    uploading = true;
    error = null;

    // Process queue sequentially
    for (const item of uploadQueue) {
      if (item.status !== "pending") continue;

      try {
        await processQueueItem(item, token);
      } catch (e) {
        // Error already handled in processQueueItem
      }
    }

    uploading = false;

    if (completedCount > 0 && errorCount === 0) {
      toastStore.push({
        variant: "success",
        message: `${completedCount} file${completedCount > 1 ? "s" : ""} uploaded`
      });
    } else if (completedCount > 0 && errorCount > 0) {
      toastStore.push({
        variant: "info",
        message: `${completedCount} uploaded, ${errorCount} failed`
      });
    }
  }

  function updateQueueItem(itemId: string, updates: Partial<QueueItem>) {
    const index = uploadQueue.findIndex(q => q.id === itemId);
    if (index === -1) return;
    Object.assign(uploadQueue[index], updates);
  }

  async function processQueueItem(item: QueueItem, token: string) {
    try {
      // Step 1: Check duplicates
      updateQueueItem(item.id, { status: "hashing" });
      const dupResult = await checkDuplicate(item.file, fetch, token);
      updateQueueItem(item.id, { hash: dupResult.hash, status: "checking" });

      if (dupResult.exists && dupResult.media) {
        updateQueueItem(item.id, {
          status: "duplicate",
          duplicateOf: {
            id: dupResult.media.id,
            title: dupResult.media.title ?? dupResult.media.originalFilename ?? "Untitled"
          }
        });
        return;
      }

      // Step 2: Create and upload
      updateQueueItem(item.id, { status: "creating" });
      const result = await createAndUpload({
        file: item.file,
        fetchFn: fetch,
        accessToken: token,
        title: item.title.trim() || null,
        onProgress: (progress: UploadProgress) => {
          updateQueueItem(item.id, { status: "uploading", progress: progress.percent });
        }
      });

      updateQueueItem(item.id, { status: "done", progress: 100, mediaId: result.mediaId });
    } catch (e) {
      updateQueueItem(item.id, {
        status: "error",
        error: e instanceof Error ? e.message : "Upload failed"
      });
    }
  }

  async function retryItem(itemId: string) {
    const item = uploadQueue.find(q => q.id === itemId);
    if (!item) return;

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    // Reset item state
    updateQueueItem(itemId, { status: "pending", progress: 0, error: undefined });

    uploading = true;
    try {
      await processQueueItem(item, token);
    } catch (e) {
      // Error handled in processQueueItem
    }
    uploading = false;
  }

  async function uploadDuplicateAnyway(itemId: string) {
    const item = uploadQueue.find(q => q.id === itemId);
    if (!item) return;

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    uploading = true;
    try {
      updateQueueItem(itemId, { status: "creating" });

      const result = await createAndUpload({
        file: item.file,
        fetchFn: fetch,
        accessToken: token,
        title: item.title.trim() || null,
        onProgress: (progress: UploadProgress) => {
          updateQueueItem(itemId, { status: "uploading", progress: progress.percent });
        }
      });

      updateQueueItem(itemId, {
        status: "done",
        progress: 100,
        mediaId: result.mediaId,
        duplicateOf: undefined,
      });

      toastStore.push({ variant: "success", message: "File uploaded" });
    } catch (e) {
      updateQueueItem(itemId, {
        status: "error",
        error: e instanceof Error ? e.message : "Upload failed"
      });
    }
    uploading = false;
  }

  function clearQueue() {
    uploadQueue = [];
    error = null;
  }

  // --- Single file mode (replace) ---

  // For replace mode, auto-set title from filename
  $effect(() => {
    if (!isBulkMode && files.length > 0 && !singleTitle) {
      const filename = files[0].file.name;
      singleTitle = filename.replace(/\.[^/.]+$/, "").replace(/[-_]/g, " ");
    }
  });

  // Validate file in single mode
  $effect(() => {
    if (!isBulkMode && files.length > 0) {
      validateSingleFile();
    }
  });

  function validateSingleFile() {
    const file = files[0]?.file;
    if (!file) return;

    if (REJECTED_VIDEO_TYPES.includes(file.type as typeof REJECTED_VIDEO_TYPES[number])) {
      error = "Video uploads are not supported. Please upload images or PDFs.";
      files = [];
      return;
    }

    if (!validateFileType(file, ALLOWED_MEDIA_TYPES)) {
      error = `Unsupported file type: ${file.type}. Supported types: Images (JPEG, PNG, GIF, WebP, SVG) and PDF.`;
      files = [];
      return;
    }

    if (file.size > MAX_FILE_SIZE) {
      error = `File is too large (${formatFileSize(file.size)}). Maximum size is ${formatFileSize(MAX_FILE_SIZE)}.`;
      files = [];
      return;
    }

    error = null;
  }

  // Single file upload progress
  type SingleUploadStage = "idle" | "hashing" | "uploading" | "finalizing" | "done";
  let singleUploadStage = $state<SingleUploadStage>("idle");
  let singleUploadProgress = $state(0);

  async function handleSingleUpload() {
    if (files.length === 0 || !replaceMediaId) return;

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    uploading = true;
    error = null;
    singleUploadProgress = 0;

    try {
      singleUploadStage = "hashing";

      await replaceUpload({
        file: files[0].file,
        mediaId: replaceMediaId,
        fetchFn: fetch,
        accessToken: token,
        onProgress: (progress: UploadProgress) => {
          singleUploadStage = "uploading";
          singleUploadProgress = progress.percent;
        }
      });

      singleUploadStage = "done";
      toastStore.push({ variant: "success", message: "File replaced" });
      await goto(`/media/${replaceMediaId}`);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to upload media";
      toastStore.push({ variant: "error", message: error });
      singleUploadStage = "idle";
    } finally {
      uploading = false;
    }
  }
</script>

<PoodlePageHeader
  title={replaceMediaId ? "Replace File" : "Upload Media"}
  backHref={replaceMediaId ? `/media/${replaceMediaId}` : "/media"}
  backLabel={replaceMediaId ? "Back to media" : "Back to library"}
/>

<div class="upload-container">
  {#if error}
    <PoodleCallout tone="danger" message={error} announceMode="polite" />
  {/if}

  {#if replaceMediaId}
    <!-- Single file replace mode -->
    {#if uploading}
      <section class="progress-section">
        <div class="progress-header">
          <span class="progress-stage">
            {singleUploadStage === "hashing" ? "Computing file hash..." :
             singleUploadStage === "uploading" ? "Uploading file..." :
             singleUploadStage === "finalizing" ? "Finalizing..." :
             singleUploadStage === "done" ? "Done!" : ""}
          </span>
          {#if singleUploadStage === "uploading"}
            <span class="progress-percent">{singleUploadProgress}%</span>
          {/if}
        </div>
        <Progress
          value={singleUploadStage === "uploading" ? singleUploadProgress : singleUploadStage === "done" ? 100 : 0}
          max={100}
          ariaLabel="Replace upload progress"
        />
        {#if singleUploadStage === "done"}
          <div class="progress-done">
            <CheckCircle size={20} />
            <span>Upload complete!</span>
          </div>
        {/if}
      </section>
    {:else}
      <section class="upload-section">
        <FileUpload
          bind:files
          accept="image/jpeg,image/png,image/gif,image/webp,image/svg+xml,application/pdf"
          maxSize={MAX_FILE_SIZE}
          disabled={uploading}
        />
        <p class="upload-hint">
          Supported formats: JPEG, PNG, GIF, WebP, SVG, PDF. Maximum size: {formatFileSize(MAX_FILE_SIZE)}.
        </p>
      </section>
    {/if}

    <div class="actions">
      <PoodleButton
        type="button"
        variant="primary"
        disabled={uploading || files.length === 0 || !!error}
        on:click={handleSingleUpload}
      >
        <svelte:fragment slot="leading">
          <Upload size={16} />
        </svelte:fragment>
        {uploading ? "Uploading..." : "Replace File"}
      </PoodleButton>
      <span class="actions-spacer"></span>
      <PoodleButton
        type="button"
        variant="ghost"
        disabled={uploading}
        on:click={() => goto(`/media/${replaceMediaId}`)}
      >
        Cancel
      </PoodleButton>
    </div>
  {:else}
    <!-- Bulk upload mode -->
    <section class="upload-section">
      <FileUpload
        bind:files
        accept="image/jpeg,image/png,image/gif,image/webp,image/svg+xml,application/pdf"
        maxSize={MAX_FILE_SIZE}
        disabled={uploading}
        multiple
      />
      <p class="upload-hint">
        Supported formats: JPEG, PNG, GIF, WebP, SVG, PDF. Maximum size: {formatFileSize(MAX_FILE_SIZE)} per file.
        <br />Select multiple files to upload them all at once.
      </p>
    </section>

    {#if uploadQueue.length > 0}
      <section class="queue-section">
        <div class="queue-header">
          <h2>Upload Queue ({uploadQueue.length} file{uploadQueue.length > 1 ? "s" : ""})</h2>
          {#if !uploading && !allDone}
            <PoodleButton type="button" variant="ghost" size="sm" on:click={clearQueue}>
              Clear All
            </PoodleButton>
          {/if}
        </div>

        <div class="queue-list">
          {#each uploadQueue as item (item.id)}
            <div class="queue-item" class:done={item.status === "done"} class:error={item.status === "error"} class:duplicate={item.status === "duplicate"}>
              <div class="queue-item-icon">
                {#if item.status === "done"}
                  <CheckCircle size={20} />
                {:else if item.status === "error"}
                  <XCircle size={20} />
                {:else if item.status === "duplicate"}
                  <AlertCircle size={20} />
                {:else}
                  <FileIcon size={20} />
                {/if}
              </div>

              <div class="queue-item-info">
                <div class="queue-item-name">{item.file.name}</div>
                <div class="queue-item-meta">
                  {formatFileSize(item.file.size)}
                  {#if item.status !== "pending" && item.status !== "done" && item.status !== "error" && item.status !== "duplicate"}
                    <span class="queue-item-status">{getStatusLabel(item.status)}</span>
                  {/if}
                  {#if item.status === "error" && item.error}
                    <span class="queue-item-error">{item.error}</span>
                  {/if}
                  {#if item.status === "duplicate" && item.duplicateOf}
                    <span class="queue-item-duplicate">
                      Duplicate of "{item.duplicateOf.title}"
                    </span>
                  {/if}
                </div>
                {#if item.status === "uploading"}
                  <Progress value={item.progress} max={100} size="sm" ariaLabel={`Upload progress for ${item.file.name}`} />
                {/if}
              </div>

              <div class="queue-item-actions">
                {#if item.status === "done" && item.mediaId}
                  <PoodleButton
                    type="button"
                    variant="ghost"
                    size="sm"
                    on:click={() => goto(`/media/${item.mediaId}`)}
                  >
                    View
                  </PoodleButton>
                {:else if item.status === "error"}
                  <PoodleButton
                    type="button"
                    variant="ghost"
                    size="sm"
                    disabled={uploading}
                    on:click={() => retryItem(item.id)}
                  >
                    Retry
                  </PoodleButton>
                {:else if item.status === "duplicate"}
                  <PoodleButton
                    type="button"
                    variant="ghost"
                    size="sm"
                    on:click={() => goto(`/media/${item.duplicateOf?.id}`)}
                  >
                    View Existing
                  </PoodleButton>
                  <PoodleButton
                    type="button"
                    variant="ghost"
                    size="sm"
                    disabled={uploading}
                    on:click={() => uploadDuplicateAnyway(item.id)}
                  >
                    Upload Anyway
                  </PoodleButton>
                {:else if item.status === "pending"}
                  <button
                    type="button"
                    class="remove-btn"
                    onclick={() => removeFromQueue(item.id)}
                    aria-label="Remove from queue"
                  >
                    <X size={16} />
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </section>
    {/if}

    <div class="actions">
      {#if allDone}
        <PoodleButton
          type="button"
          variant="primary"
          on:click={() => goto("/media")}
        >
          Done
        </PoodleButton>
      {:else}
        <PoodleButton
          type="button"
          variant="primary"
          disabled={uploading || uploadQueue.length === 0 || pendingCount === 0}
          on:click={handleUpload}
        >
          <svelte:fragment slot="leading">
            <Upload size={16} />
          </svelte:fragment>
          {uploading ? "Uploading..." : `Upload ${pendingCount} file${pendingCount > 1 ? "s" : ""}`}
        </PoodleButton>
      {/if}
      <span class="actions-spacer"></span>
      <PoodleButton
        type="button"
        variant="ghost"
        disabled={uploading}
        on:click={() => goto("/media")}
      >
        Cancel
      </PoodleButton>
    </div>
  {/if}
</div>

<style>
  .upload-container {
    max-width: 48rem;
    margin-top: 1.5rem;
    margin-inline: auto;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;

    /* Override FileUpload component tokens for dark theme */
    --underlay-upload-border: 2px dashed var(--admin-color-border-strong);
    --underlay-upload-border-active: 2px dashed var(--admin-color-accent);
    --underlay-upload-bg: var(--admin-color-surface-card);
    --underlay-upload-bg-hover: var(--admin-color-surface-subtle);
    --color-border: var(--admin-color-border-subtle);
    --color-surface: var(--admin-color-surface-card);
    --color-surface-subtle: var(--admin-color-surface-subtle);
    --color-surface-hover: var(--admin-color-surface-subtle);
    --color-text-muted: var(--admin-color-text-muted);
    --color-primary: var(--admin-color-accent);
  }

  .upload-section {
    background: var(--admin-color-surface-card);
    border: 1px solid var(--admin-color-border-subtle);
    border-radius: 0.5rem;
    padding: 1.5rem;
  }

  .upload-hint {
    margin: 1rem 0 0;
    font-size: 0.875rem;
    color: var(--admin-color-text-muted);
    line-height: 1.5;
  }

  .progress-section {
    background: var(--admin-color-surface-card);
    border: 1px solid var(--admin-color-border-subtle);
    border-radius: 0.5rem;
    padding: 1.5rem;
  }

  .progress-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .progress-stage {
    font-size: 0.875rem;
    color: var(--admin-color-text);
  }

  .progress-percent {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--admin-color-text);
  }

  .progress-done {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 1rem;
    color: #10b981;
    font-weight: 500;
  }

  .queue-section {
    background: var(--admin-color-surface-card);
    border: 1px solid var(--admin-color-border-subtle);
    border-radius: 0.5rem;
    padding: 1.5rem;
  }

  .queue-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .queue-header h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--admin-color-text);
  }

  .queue-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .queue-item {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--admin-color-surface-subtle);
    border-radius: 0.5rem;
  }

  .queue-item.done {
    background: rgba(16, 185, 129, 0.1);
  }

  .queue-item.done .queue-item-icon {
    color: #10b981;
  }

  .queue-item.error {
    background: rgba(239, 68, 68, 0.1);
  }

  .queue-item.error .queue-item-icon {
    color: #ef4444;
  }

  .queue-item.duplicate {
    background: rgba(245, 158, 11, 0.1);
  }

  .queue-item.duplicate .queue-item-icon {
    color: #f59e0b;
  }

  .queue-item-icon {
    flex-shrink: 0;
    color: var(--admin-color-text-muted);
    padding-top: 0.125rem;
  }

  .queue-item-info {
    flex: 1;
    min-width: 0;
  }

  .queue-item-name {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--admin-color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .queue-item-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 0.25rem;
    font-size: 0.75rem;
    color: var(--admin-color-text-muted);
  }

  .queue-item-status {
    color: var(--admin-color-accent);
    font-weight: 500;
  }

  .queue-item-error {
    color: #ef4444;
  }

  .queue-item-duplicate {
    color: #fbbf24;
  }

  .queue-item-actions {
    display: flex;
    gap: 0.5rem;
    flex-shrink: 0;
    align-items: center;
  }

  .remove-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.75rem;
    height: 1.75rem;
    border: none;
    background: transparent;
    color: var(--admin-color-text-muted);
    border-radius: 0.25rem;
    cursor: pointer;
    transition: background-color 0.15s, color 0.15s;
  }

  .remove-btn:hover {
    background: var(--admin-color-surface-subtle);
    color: #ef4444;
  }

  .actions {
    display: flex;
    gap: 0.75rem;
  }

  .actions-spacer {
    flex: 1;
  }
</style>
