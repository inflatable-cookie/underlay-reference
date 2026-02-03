<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { mediaCommands, detectMediaKindFromMimeType } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import { PageHeader, useToasts } from "@decodelabs/underlay/patterns";
  import { Button, PageLoading, FormError, Field, TextInput, FileUpload, type FileUploadItem } from "@decodelabs/underlay/components";
  import Upload from "lucide-svelte/icons/upload";

  const toastStore = useToasts();

  // Check if we're replacing an existing media
  const replaceMediaId = $derived($page.url.searchParams.get("replace"));

  let files = $state<FileUploadItem[]>([]);
  let title = $state("");
  let altText = $state("");
  let uploading = $state(false);
  let error = $state<string | null>(null);

  // Auto-set title from filename
  $effect(() => {
    if (files.length > 0 && !title) {
      const filename = files[0].file.name;
      title = filename.replace(/\.[^/.]+$/, "").replace(/[-_]/g, " ");
    }
  });

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

    try {
      const file = files[0].file;

      // Step 1: Create media record (or use existing for replace)
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

      // Step 2: Initiate upload to get pre-signed URL
      const uploadInfo = await mediaCommands.initiateUpload(
        mediaId,
        {
          contentType: file.type,
          contentLength: file.size
        },
        fetch,
        token
      );

      // Step 3: Upload file to blob storage
      const uploadResponse = await fetch(uploadInfo.uploadPlan.uploadUrl, {
        method: uploadInfo.uploadPlan.method,
        body: file,
        headers: {
          "Content-Type": file.type,
          ...uploadInfo.uploadPlan.headers
        }
      });

      if (!uploadResponse.ok) {
        throw new Error("Failed to upload file to storage");
      }

      // Step 4: Finalise upload
      await mediaCommands.finaliseUpload(
        mediaId,
        uploadInfo.versionId,
        { sha256: "" }, // Hash verification is optional
        fetch,
        token
      );

      toastStore.push({ variant: "success", message: replaceMediaId ? "File replaced" : "Media uploaded" });
      await goto(`/media/${mediaId}`);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to upload media";
      toastStore.push({ variant: "error", message: error });
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

  <section class="upload-section">
    <FileUpload
      bind:files
      accept="image/*,video/*,audio/*,.pdf,.doc,.docx,.xls,.xlsx,.ppt,.pptx,.txt"
      maxSize={100 * 1024 * 1024}
      disabled={uploading}
    />
  </section>

  {#if files.length > 0 && !replaceMediaId}
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
      disabled={uploading || files.length === 0}
    >
      <Upload size={16} />
      {uploading ? "Uploading..." : replaceMediaId ? "Replace File" : "Upload"}
    </Button>
  </div>
</div>

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
</style>
