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

  const toastStore = useToasts();

  // Check if we're replacing an existing media
  const replaceMediaId = $derived($page.url.searchParams.get("replace"));

  // State
  let files = $state<FileUploadItem[]>([]);
  let title = $state("");
  let altText = $state("");
  let uploading = $state(false);
  let error = $state<string | null>(null);

  // Upload progress state
  type UploadStage = "idle" | "hashing" | "checking" | "creating" | "uploading" | "finalizing" | "done";
  let uploadStage = $state<UploadStage>("idle");
  let uploadProgress = $state(0);
  let fileHash = $state<string | null>(null);

  // Deduplication dialog state
  let showDuplicateDialog = $state(false);
  let duplicateMedia = $state<{ id: string; title: string } | null>(null);

  // File size limit (50MB)
  const MAX_FILE_SIZE = 50 * 1024 * 1024;

  // Auto-set title from filename
  $effect(() => {
    if (files.length > 0 && !title) {
      const filename = files[0].file.name;
      title = filename.replace(/\.[^/.]+$/, "").replace(/[-_]/g, " ");
    }
  });

  // Validate file when selected
  $effect(() => {
    if (files.length > 0) {
      validateSelectedFile();
    }
  });

  function validateSelectedFile() {
    const file = files[0]?.file;
    if (!file) return;

    // Check for video types (explicitly rejected)
    if (REJECTED_VIDEO_TYPES.includes(file.type as typeof REJECTED_VIDEO_TYPES[number])) {
      error = "Video uploads are not supported. Please upload images or PDFs.";
      files = [];
      return;
    }

    // Check file type
    if (!validateFileType(file, ALLOWED_MEDIA_TYPES)) {
      error = `Unsupported file type: ${file.type}. Supported types: Images (JPEG, PNG, GIF, WebP, SVG) and PDF.`;
      files = [];
      return;
    }

    // Check file size
    if (file.size > MAX_FILE_SIZE) {
      error = `File is too large (${formatFileSize(file.size)}). Maximum size is ${formatFileSize(MAX_FILE_SIZE)}.`;
      files = [];
      return;
    }

    error = null;
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function getStageLabel(stage: UploadStage): string {
    switch (stage) {
      case "hashing": return "Computing file hash...";
      case "checking": return "Checking for duplicates...";
      case "creating": return "Creating media record...";
      case "uploading": return "Uploading file...";
      case "finalizing": return "Finalizing...";
      case "done": return "Done!";
      default: return "";
    }
  }

  async function handleUpload() {
    if (files.length === 0) {
      error = "Please select a file to upload";
      return;
    }

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    uploading = true;
    error = null;
    uploadProgress = 0;

    try {
      const file = files[0].file;

      // Step 1: Compute file hash for deduplication
      uploadStage = "hashing";
      fileHash = await computeFileHash(file);

      // Step 2: Check for duplicates (skip for replace mode)
      if (!replaceMediaId) {
        uploadStage = "checking";
        const duplicateCheck = await mediaCommands.checkDuplicate(
          { sha256: fileHash },
          fetch,
          token
        );

        if (duplicateCheck.exists && duplicateCheck.media) {
          // Show duplicate dialog
          duplicateMedia = {
            id: duplicateCheck.media.id,
            title: duplicateCheck.media.title ?? duplicateCheck.media.originalFilename ?? "Untitled"
          };
          showDuplicateDialog = true;
          uploading = false;
          uploadStage = "idle";
          return;
        }
      }

      await continueUpload(file, token);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to upload media";
      toastStore.push({ variant: "error", message: error });
      uploadStage = "idle";
    } finally {
      uploading = false;
    }
  }

  async function continueUpload(file: File, token: string) {
    try {
      // Step 3: Create media record (or use existing for replace)
      uploadStage = "creating";
      let mediaId: string;
      if (replaceMediaId) {
        mediaId = replaceMediaId;
      } else {
        const kind = detectMediaKindFromMimeType(file.type);
        const mediaRecord = await mediaCommands.createMedia(
          {
            kind,
            visibility: "public",
            originalFilename: file.name,
            title: title.trim() || null
          },
          fetch,
          token
        );
        mediaId = mediaRecord.id;
      }

      // Step 4: Initiate upload to get pre-signed URL
      const uploadInfo = await mediaCommands.initiateUpload(
        mediaId,
        {
          contentType: file.type,
          contentLength: file.size
        },
        fetch,
        token
      );

      // Step 5: Upload file with progress tracking
      uploadStage = "uploading";

      // Convert MediaUploadPlan to UploadPlan format expected by uploadToBlob
      const plan = {
        uploadUrl: uploadInfo.uploadPlan.uploadUrl,
        method: uploadInfo.uploadPlan.method,
        requiredHeaders: uploadInfo.uploadPlan.headers,
        maxBytes: uploadInfo.uploadPlan.maxBytes ?? MAX_FILE_SIZE,
        allowedContentTypes: uploadInfo.uploadPlan.allowedContentTypes ?? [],
        expiresAt: uploadInfo.uploadPlan.expiresAt,
        objectKey: "" // Not needed for upload, but required by type
      };

      await uploadToBlob(plan, file, {
        onProgress: (progress: UploadProgress) => {
          uploadProgress = progress.percent;
        }
      });

      // Step 6: Finalise upload with hash verification
      uploadStage = "finalizing";
      await mediaCommands.finaliseUpload(
        mediaId,
        uploadInfo.versionId,
        { sha256: fileHash ?? "" },
        fetch,
        token
      );

      uploadStage = "done";
      toastStore.push({ variant: "success", message: replaceMediaId ? "File replaced" : "Media uploaded" });
      await goto(`/media/${mediaId}`);
    } catch (e) {
      throw e;
    }
  }

  function handleUseDuplicate() {
    if (duplicateMedia) {
      goto(`/media/${duplicateMedia.id}`);
    }
    showDuplicateDialog = false;
    duplicateMedia = null;
  }

  async function handleUploadAnyway() {
    showDuplicateDialog = false;
    duplicateMedia = null;

    const token = auth.getToken();
    if (!token || files.length === 0) return;

    uploading = true;
    try {
      await continueUpload(files[0].file, token);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to upload media";
      toastStore.push({ variant: "error", message: error });
      uploadStage = "idle";
    } finally {
      uploading = false;
    }
  }

  function handleCancelDuplicate() {
    showDuplicateDialog = false;
    duplicateMedia = null;
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

  {#if uploading}
    <section class="progress-section">
      <div class="progress-header">
        <span class="progress-stage">{getStageLabel(uploadStage)}</span>
        {#if uploadStage === "uploading"}
          <span class="progress-percent">{uploadProgress}%</span>
        {/if}
      </div>
      <ProgressBar value={uploadStage === "uploading" ? uploadProgress : uploadStage === "done" ? 100 : 0} max={100} />
      {#if uploadStage === "done"}
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

  {#if files.length > 0 && !replaceMediaId && !uploading}
    <section class="metadata-section">
      <h2>Details</h2>
      <div class="form-fields">
        <Field label="Title">
          <TextInput
            bind:value={title}
            placeholder="Enter title"
            disabled={uploading}
          />
        </Field>
        <Field label="Alt Text" hint="Describe the content for accessibility">
          <TextInput
            bind:value={altText}
            placeholder="Describe the image or document"
            disabled={uploading}
          />
        </Field>
      </div>
    </section>
  {/if}

  <div class="actions">
    <Button
      type="button"
      variant="subtle"
      onclick={() => goto(replaceMediaId ? `/media/${replaceMediaId}` : "/media")}
      disabled={uploading}
    >
      Cancel
    </Button>
    <Button
      type="button"
      variant="primary"
      onclick={handleUpload}
      disabled={uploading || files.length === 0 || !!error}
    >
      <Upload size={16} />
      {uploading ? "Uploading..." : replaceMediaId ? "Replace File" : "Upload"}
    </Button>
  </div>
</div>

{#if showDuplicateDialog}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="dialog-backdrop" onclick={handleCancelDuplicate} role="presentation">
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div class="dialog" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
      <div class="dialog-icon">
        <AlertCircle size={32} />
      </div>
      <h2 class="dialog-title">Duplicate File Detected</h2>
      <p class="dialog-message">
        A file with the same content already exists in your library:
      </p>
      {#if duplicateMedia}
        <p class="dialog-existing">
          <strong>{duplicateMedia.title}</strong>
        </p>
      {/if}
      <div class="dialog-actions">
        <Button type="button" variant="subtle" onclick={handleCancelDuplicate}>
          Cancel
        </Button>
        <Button type="button" variant="secondary" onclick={handleUploadAnyway}>
          Upload Anyway
        </Button>
        <Button type="button" variant="primary" onclick={handleUseDuplicate}>
          Use Existing
        </Button>
      </div>
    </div>
  </div>
{/if}

<style>
  .upload-container {
    max-width: 40rem;
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

  .metadata-section {
    background: var(--bg-surface, #fff);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 0.5rem;
    padding: 1.5rem;
  }

  .metadata-section h2 {
    margin: 0 0 1rem;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary, #111827);
  }

  .form-fields {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
  }

  .dialog-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .dialog {
    background: var(--bg-surface, #fff);
    border-radius: 0.75rem;
    padding: 1.5rem;
    max-width: 400px;
    width: 90%;
    text-align: center;
    box-shadow: 0 25px 50px rgba(0, 0, 0, 0.25);
  }

  .dialog-icon {
    color: var(--warning, #f59e0b);
    margin-bottom: 1rem;
  }

  .dialog-title {
    margin: 0 0 0.5rem;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text-primary, #111827);
  }

  .dialog-message {
    margin: 0 0 0.5rem;
    font-size: 0.875rem;
    color: var(--text-secondary, #6b7280);
  }

  .dialog-existing {
    margin: 0 0 1.5rem;
    padding: 0.75rem;
    background: var(--bg-muted, #f3f4f6);
    border-radius: 0.5rem;
    font-size: 0.875rem;
  }

  .dialog-actions {
    display: flex;
    justify-content: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }
</style>
