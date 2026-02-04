<!--
  Re-export of Underlay LogList with acme-client types.
  This file maintains a local wrapper for dashboard usage.
-->
<script lang="ts">
  import type { ActivityEntry } from "acme-client";
  import { LogList } from "@decodelabs/underlay/components";

  interface Props {
    activities: ActivityEntry[];
    loading?: boolean;
    error?: string | null;
    emptyMessage?: string;
  }

  let { activities, loading = false, error = null, emptyMessage = "No activity yet" }: Props = $props();

  const entries = $derived(
    activities.map((activity) => ({
      id: activity.id,
      occurredAt: activity.occurredAt,
      actor: activity.actor
        ? {
            id: activity.actor.id,
            email: activity.actor.email,
            name: activity.actor.displayName ?? undefined
          }
        : undefined,
      action: activity.action,
      resourceType: activity.resourceType,
      resourceId: activity.resourceId,
      resourceLabel: (activity.details?.resourceLabel as string | undefined) ?? undefined,
      details: activity.details
    }))
  );
</script>

<LogList entries={entries} {loading} {error} {emptyMessage} />
