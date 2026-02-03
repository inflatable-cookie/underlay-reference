<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { mediaCommands, detectMediaKindFromMimeType } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import {
    PageHeader,
    useToasts,
    computeFileHash,
    uploadToBlob,
    validateFileType,
    ALLOWED_MEDIA_TYPES,
    REJECTED_VIDEO_TYPES,
    type UploadProgress
  } from "@decodelabs/underlay/patterns";
  import { Button, PageLoading, FormError, Field, TextInput, FileUpload, ProgressBar, type FileUploadItem } from "@decodelabs/underlay/components";
  import Upload from "lucide-svelte/icons/upload";
  import AlertCircle from "lucide-svelte/icons/alert-circle";
  import CheckCircle from "lucide-svelte/icons/check-circle";
  import XCircle from "lucide-svelte/icons/x-circle";
  import X from "lucide-svelte/icons/x";
  import FileIcon from "lucide-svelte/icons/file";

  const toastStore = useToasts();

  // Check if we're replacing an existing media (single file mode)
  const replaceMediaId = $derived($page.url.searchParams.get("replace"));
  const isBulkMode = $derived(!replaceMediaId);

  // File size limit (50MB)
  const MAX_FILE_SIZE = 50 * 1024 * 1024;

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

  // Deduplication dialog state
  let showDuplicateDialog = $state(false);
  let duplicateItem = $state<QueueItem | null>(null);

  // Derived states
  const hasFiles = $derived(files.length > 0 || uploadQueue.length > 0);
  const pendingCount = $derived(uploadQueue.filter(q => q.status === "pending").length);
  const completedCount = $derived(uploadQueue.filter(q => q.status === "done").length);
  const errorCount = $derived(uploadQueue.filter(q => q.status === "error").length);
  const allDone = $derived(uploadQueue.length > 0 && uploadQueue.every(q => q.status === "done" || q.status === "error" || q.status === "duplicate"));

  // Add files to queue when selected
  $effect(() => {
    if (files.length > 0) {
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

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
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

  async function processQueueItem(item: QueueItem, token: string) {
    const index = uploadQueue.findIndex(q => q.id === item.id);
    if (index === -1) return;

    try {
      // Step 1: Hash
      uploadQueue[index].status = "hashing";
      const hash = await computeFileHash(item.file);
      uploadQueue[index].hash = hash;

      // Step 2: Check duplicates
      uploadQueue[index].status = "checking";
      const duplicateCheck = await mediaCommands.checkDuplicate(
        { sha256: hash },
        fetch,
        token
      );

      if (duplicateCheck.exists && duplicateCheck.media) {
        uploadQueue[index].status = "duplicate";
        uploadQueue[index].duplicateOf = {
          id: duplicateCheck.media.id,
          title: duplicateCheck.media.title ?? duplicateCheck.media.originalFilename ?? "Untitled"
        };
        return;
      }

      // Step 3: Create media
      uploadQueue[index].status = "creating";
      const kind = detectMediaKindFromMimeType(item.file.type);
      const mediaRecord = await mediaCommands.createMedia(
        {
          kind,
          visibility: "public",
          originalFilename: item.file.name,
          title: item.title.trim() || null
        },
        fetch,
        token
      );
      uploadQueue[index].mediaId = mediaRecord.id;

      // Step 4: Initiate upload
      const uploadInfo = await mediaCommands.initiateUpload(
        mediaRecord.id,
        {
          contentType: item.file.type,
          contentLength: item.file.size
        },
        fetch,
        token
      );

      // Step 5: Upload with progress
      uploadQueue[index].status = "uploading";

      const plan = {
        uploadUrl: uploadInfo.uploadPlan.uploadUrl,
        method: uploadInfo.uploadPlan.method,
        requiredHeaders: uploadInfo.uploadPlan.headers,
        maxBytes: uploadInfo.uploadPlan.maxBytes ?? MAX_FILE_SIZE,
        allowedContentTypes: uploadInfo.uploadPlan.allowedContentTypes ?? [],
        expiresAt: uploadInfo.uploadPlan.expiresAt,
        objectKey: ""
      };

      await uploadToBlob(plan, item.file, {
        onProgress: (progress: UploadProgress) => {
          const idx = uploadQueue.findIndex(q => q.id === item.id);
          if (idx !== -1) {
            uploadQueue[idx].progress = progress.percent;
          }
        }
      });

      // Step 6: Finalize
      uploadQueue[index].status = "finalizing";
      await mediaCommands.finaliseUpload(
        mediaRecord.id,
        uploadInfo.versionId,
        { sha256: hash },
        fetch,
        token
      );

      uploadQueue[index].status = "done";
      uploadQueue[index].progress = 100;
    } catch (e) {
      const idx = uploadQueue.findIndex(q => q.id === item.id);
      if (idx !== -1) {
        uploadQueue[idx].status = "error";
        uploadQueue[idx].error = e instanceof Error ? e.message : "Upload failed";
      }
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
    const index = uploadQueue.findIndex(q => q.id === itemId);
    uploadQueue[index].status = "pending";
    uploadQueue[index].progress = 0;
    uploadQueue[index].error = undefined;

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

    const index = uploadQueue.findIndex(q => q.id === itemId);

    uploading = true;
    try {
      // Skip duplicate check, go straight to create
      uploadQueue[index].status = "creating";
      const kind = detectMediaKindFromMimeType(item.file.type);
      const mediaRecord = await mediaCommands.createMedia(
        {
          kind,
          visibility: "public",
          originalFilename: item.file.name,
          title: item.title.trim() || null
        },
        fetch,
        token
      );
      uploadQueue[index].mediaId = mediaRecord.id;

      // Continue with upload
      const uploadInfo = await mediaCommands.initiateUpload(
        mediaRecord.id,
        {
          contentType: item.file.type,
          contentLength: item.file.size
        },
        fetch,
        token
      );

      uploadQueue[index].status = "uploading";

      const plan = {
        uploadUrl: uploadInfo.uploadPlan.uploadUrl,
        method: uploadInfo.uploadPlan.method,
        requiredHeaders: uploadInfo.uploadPlan.headers,
        maxBytes: uploadInfo.uploadPlan.maxBytes ?? MAX_FILE_SIZE,
        allowedContentTypes: uploadInfo.uploadPlan.allowedContentTypes ?? [],
        expiresAt: uploadInfo.uploadPlan.expiresAt,
        objectKey: ""
      };

      await uploadToBlob(plan, item.file, {
        onProgress: (progress: UploadProgress) => {
          const idx = uploadQueue.findIndex(q => q.id === item.id);
          if (idx !== -1) {
            uploadQueue[idx].progress = progress.percent;
          }
        }
      });

      uploadQueue[index].status = "finalizing";
      await mediaCommands.finaliseUpload(
        mediaRecord.id,
        uploadInfo.versionId,
        { sha256: item.hash ?? "" },
        fetch,
        token
      );

      uploadQueue[index].status = "done";
      uploadQueue[index].progress = 100;
      uploadQueue[index].duplicateOf = undefined;

      toastStore.push({ variant: "success", message: "File uploaded" });
    } catch (e) {
      uploadQueue[index].status = "error";
      uploadQueue[index].error = e instanceof Error ? e.message : "Upload failed";
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
      const file = files[0].file;

      // Hash
      singleUploadStage = "hashing";
      const hash = await computeFileHash(file);

      // Initiate
      const uploadInfo = await mediaCommands.initiateUpload(
        replaceMediaId,
        {
          contentType: file.type,
          contentLength: file.size
        },
        fetch,
        token
      );

      // Upload
      singleUploadStage = "uploading";
      const plan = {
        uploadUrl: uploadInfo.uploadPlan.uploadUrl,
        method: uploadInfo.uploadPlan.method,
        requiredHeaders: uploadInfo.uploadPlan.headers,
        maxBytes: uploadInfo.uploadPlan.maxBytes ?? MAX_FILE_SIZE,
        allowedContentTypes: uploadInfo.uploadPlan.allowedContentTypes ?? [],
        expiresAt: uploadInfo.uploadPlan.expiresAt,
        objectKey: ""
      };

      await uploadToBlob(plan, file, {
        onProgress: (progress: UploadProgress) => {
          singleUploadProgress = progress.percent;
        }
      });

      // Finalize
      singleUploadStage = "finalizing";
      await mediaCommands.finaliseUpload(
        replaceMediaId,
        uploadInfo.versionId,
        { sha256: hash },
        fetch,
        token
      );

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

<PageHeader
  title={replaceMediaId ? "Replace File" : "Upload Media"}
  backHref={replaceMediaId ? `/media/${replaceMediaId}` : "/media"}
  backLabel={replaceMediaId ? "Back to media" : "Back to library"}
/>

<div class="upload-container">
  {#if error}
    <FormError message={error} />
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
        <ProgressBar value={singleUploadStage === "uploading" ? singleUploadProgress : singleUploadStage === "done" ? 100 : 0} max={100} />
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
      <Button
        type="button"
        variant="subtle"
        onclick={() => goto(`/media/${replaceMediaId}`)}
        disabled={uploading}
      >
        Cancel
      </Button>
      <Button
        type="button"
        variant="primary"
        onclick={handleSingleUpload}
        disabled={uploading || files.length === 0 || !!error}
      >
        <Upload size={16} />
        {uploading ? "Uploading..." : "Replace File"}
      </Button>
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
            <Button type="button" variant="subtle" onclick={clearQueue} size="sm">
              Clear All
            </Button>
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
                  <ProgressBar value={item.progress} max={100} size="sm" />
                {/if}
              </div>

              <div class="queue-item-actions">
                {#if item.status === "done" && item.mediaId}
                  <Button
                    type="button"
                    variant="subtle"
                    size="sm"
                    onclick={() => goto(`/media/${item.mediaId}`)}
                  >
                    View
                  </Button>
                {:else if item.status === "error"}
                  <Button
                    type="button"
                    variant="subtle"
                    size="sm"
                    onclick={() => retryItem(item.id)}
                    disabled={uploading}
                  >
                    Retry
                  </Button>
                {:else if item.status === "duplicate"}
                  <Button
                    type="button"
                    variant="subtle"
                    size="sm"
                    onclick={() => goto(`/media/${item.duplicateOf?.id}`)}
                  >
                    View Existing
                  </Button>
                  <Button
                    type="button"
                    variant="subtle"
                    size="sm"
                    onclick={() => uploadDuplicateAnyway(item.id)}
                    disabled={uploading}
                  >
                    Upload Anyway
                  </Button>
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
      <Button
        type="button"
        variant="subtle"
        onclick={() => goto("/media")}
        disabled={uploading}
      >
        Cancel
      </Button>
      {#if allDone}
        <Button
          type="button"
          variant="primary"
          onclick={() => goto("/media")}
        >
          Done
        </Button>
      {:else}
        <Button
          type="button"
          variant="primary"
          onclick={handleUpload}
          disabled={uploading || uploadQueue.length === 0 || pendingCount === 0}
        >
          <Upload size={16} />
          {uploading ? "Uploading..." : `Upload ${pendingCount} file${pendingCount > 1 ? "s" : ""}`}
        </Button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .upload-container {
    max-width: 48rem;
    margin-top: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .upload-section {
    background: var(--bg-surface, #fff);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 0.5rem;
    padding: 1.5rem;
  }

  .upload-hint {
    margin: 1rem 0 0;
    font-size: 0.875rem;
    color: var(--text-secondary, #6b7280);
    line-height: 1.5;
  }

  .progress-section {
    background: var(--bg-surface, #fff);
    border: 1px solid var(--border-color, #e5e7eb);
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
    color: var(--text-primary, #111827);
  }

  .progress-percent {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-primary, #111827);
  }

  .progress-done {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 1rem;
    color: var(--success, #10b981);
    font-weight: 500;
  }

  .queue-section {
    background: var(--bg-surface, #fff);
    border: 1px solid var(--border-color, #e5e7eb);
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
    color: var(--text-primary, #111827);
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
    background: var(--bg-muted, #f3f4f6);
    border-radius: 0.5rem;
  }

  .queue-item.done {
    background: var(--success-bg, #ecfdf5);
  }

  .queue-item.done .queue-item-icon {
    color: var(--success, #10b981);
  }

  .queue-item.error {
    background: var(--danger-bg, #fef2f2);
  }

  .queue-item.error .queue-item-icon {
    color: var(--danger, #ef4444);
  }

  .queue-item.duplicate {
    background: var(--warning-bg, #fffbeb);
  }

  .queue-item.duplicate .queue-item-icon {
    color: var(--warning, #f59e0b);
  }

  .queue-item-icon {
    flex-shrink: 0;
    color: var(--text-secondary, #6b7280);
    padding-top: 0.125rem;
  }

  .queue-item-info {
    flex: 1;
    min-width: 0;
  }

  .queue-item-name {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-primary, #111827);
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
    color: var(--text-secondary, #6b7280);
  }

  .queue-item-status {
    color: var(--primary, #3b82f6);
    font-weight: 500;
  }

  .queue-item-error {
    color: var(--danger, #ef4444);
  }

  .queue-item-duplicate {
    color: var(--warning-text, #b45309);
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
    color: var(--text-secondary, #6b7280);
    border-radius: 0.25rem;
    cursor: pointer;
    transition: background-color 0.15s, color 0.15s;
  }

  .remove-btn:hover {
    background: var(--bg-hover, #e5e7eb);
    color: var(--danger, #ef4444);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
  }
</style>
