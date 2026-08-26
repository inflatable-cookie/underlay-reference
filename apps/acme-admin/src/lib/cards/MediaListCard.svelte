<script lang="ts">
  import { MediaListCard as UnderlayMediaListCard } from "@inflatable-cookie/underlay/templates";
  import type { MediaSummary } from "@api-client";

  interface Props {
    media: MediaSummary;
    selectionMode?: boolean;
    reorderMode?: boolean;
    selected?: boolean;
    onSelectionChange?: (mediaId: string, selected: boolean) => void;
    onDelete?: (mediaId: string) => void;
    onCopyId?: (mediaId: string) => void | Promise<void>;
  }

  let props: Props = $props();
</script>

<UnderlayMediaListCard {...props} />
