<!--
  Re-export of Underlay ActivityFeed with acme-client types.
  This file maintains backwards compatibility with existing imports.
-->
<script lang="ts">
  import type { ActivityEntry } from "acme-client";
  import { ActivityFeed } from "@decodelabs/underlay/components";

  interface Props {
    activities: ActivityEntry[];
    loading?: boolean;
    error?: string | null;
    emptyMessage?: string;
  }

  let { activities, loading = false, error = null, emptyMessage = "No activity yet" }: Props = $props();
</script>

<ActivityFeed {activities} {loading} {error} {emptyMessage} />
