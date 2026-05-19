<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";
  import { MediaUploadWorkflowPage } from "@decodelabs/underlay/templates";
  import { auth } from "$lib/stores/auth";
  import {
    MAX_FILE_SIZE,
    checkDuplicate,
    createAndUpload,
    replaceUpload
  } from "$lib/utils/upload-pipeline";

  const toastStore = useToasts();
  const replaceMediaId = $derived($page.url.searchParams.get("replace"));

  function requireToken(): string {
    const token = auth.getToken();
    if (!token) {
      throw new Error("Not authenticated");
    }
    return token;
  }

  function notify(variant: "success" | "info" | "error", message: string): void {
    toastStore.push({ variant, message });
  }
</script>

<MediaUploadWorkflowPage
  {replaceMediaId}
  maxFileSize={MAX_FILE_SIZE}
  checkDuplicate={(file) => checkDuplicate(file, fetch, requireToken())}
  createUpload={({ file, title, onProgress }) =>
    createAndUpload({
      file,
      fetchFn: fetch,
      accessToken: requireToken(),
      title,
      onProgress
    })}
  replaceUpload={({ file, mediaId, onProgress }) =>
    replaceUpload({
      file,
      mediaId,
      fetchFn: fetch,
      accessToken: requireToken(),
      onProgress
    })}
  navigate={(href) => goto(href)}
  onToast={notify}
/>
