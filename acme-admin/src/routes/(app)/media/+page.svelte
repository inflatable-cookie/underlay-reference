<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import {
    FilterBar,
    PageHeader,
    useToasts,
    useAuthenticatedData,
    getMediaKindLabel,
    MediaKind
  } from "@decodelabs/underlay/patterns";
  import {
    Button,
    Field,
    FormError,
    ListGrid,
    ListCard,
    Badge,
    OrderBy,
    PageLoading,
    Select,
    TextInput,
    type OrderByValue
  } from "@decodelabs/underlay/components";
  import { gotoWithContext } from "@decodelabs/underlay/client";
  import { mediaCommands, type MediaSummary } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import Plus from "lucide-svelte/icons/plus";
  import Image from "lucide-svelte/icons/image";
  import FileText from "lucide-svelte/icons/file-text";
  import Film from "lucide-svelte/icons/film";
  import Music from "lucide-svelte/icons/music";
  import FileIcon from "lucide-svelte/icons/file";
  import Trash2 from "lucide-svelte/icons/trash-2";

  const toastStore = useToasts();

  // Fetch media using authenticated data pattern
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const items = await mediaCommands.listMediaAdmin(fetch, token);
      return { items };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { items: [] as MediaSummary[] }
    }
  );

  // Trigger fetch when auth is ready
  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  async function handleDeleteMedia(mediaId: string) {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await mediaCommands.softDeleteMedia(mediaId, fetch, token);
      toastStore.push({ variant: "success", message: "Media moved to trash" });
      await pageData.refetch();
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

  function getKindVariant(kind: string): "default" | "success" | "warning" | "danger" | "info" | "muted" {
    switch (kind) {
      case MediaKind.Image:
        return "info";
      case MediaKind.Video:
        return "warning";
      case MediaKind.Audio:
        return "success";
      case MediaKind.Document:
      case MediaKind.Pdf:
        return "muted";
      default:
        return "default";
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
</script>

<PageHeader title="Media Library" backHref="/" backLabel="Back to dashboard">
  {#snippet actions()}
    <Button
      type="button"
      variant="subtle"
      onclick={() => goto("/media/trash")}
    >
      <Trash2 size={16} />
      Trash
    </Button>
    <Button
      type="button"
      variant="primary"
      onclick={() =>
        void gotoWithContext("/media/upload", {
          label: "Media",
          href: "/media",
          type: "list"
        })}
    >
      <Plus size={16} />
      Upload
    </Button>
  {/snippet}
</PageHeader>

{#if pageData.loading}
  <PageLoading message="Loading media..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if (pageData.data?.items ?? []).length === 0}
  <p class="empty-state">No media found. Upload your first file to get started.</p>
{:else}
  <ListGrid minItemWidth={20}>
    {#each pageData.data?.items ?? [] as item}
      {@const KindIcon = getKindIcon(item.kind)}
      <ListCard
        title={item.title ?? item.originalFilename ?? "Untitled"}
        subtitle={formatFileSize(item.byteSize)}
        href={`/media/${item.id}`}
      >
        {#snippet media()}
          <KindIcon size={20} />
        {/snippet}
        {#snippet titleSuffix()}
          <Badge variant={getKindVariant(item.kind)} size="sm">
            {getMediaKindLabel(item.kind)}
          </Badge>
        {/snippet}
      </ListCard>
    {/each}
  </ListGrid>
{/if}

<style>
  .empty-state {
    padding: 2rem;
    text-align: center;
    color: var(--text-secondary, #6b7280);
  }
</style>
