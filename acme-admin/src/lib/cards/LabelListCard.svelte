<script lang="ts">
  import { EntityListCard } from "@inflatable-cookie/underlay/templates";
  import { gotoWithContext } from "@inflatable-cookie/underlay/client/navigation";
  import type { NavigationContext } from "@inflatable-cookie/underlay/runtime/navigation";
  import { TimeAgo } from "@inflatable-cookie/poodle-svelte";
  import type { Label } from "@api-client";

  interface Props {
    label: Label;
    sourceContext?: NavigationContext;
  }

  let { label, sourceContext }: Props = $props();

  function handleOpen(): void {
    void gotoWithContext(
      `/projects/${label.projectId}/labels/${label.id}`,
      sourceContext ?? {
        label: "Labels",
        href: `/projects/${label.projectId}/labels`,
        type: "list"
      }
    );
  }
</script>

{#snippet footer()}
  <TimeAgo datetime={label.createdAt} tooltipFormat="datetime" typography="inherit" />
{/snippet}

<EntityListCard
  title={label.name}
  accentColor={label.color || "#6366f1"}
  {footer}
  onClick={handleOpen}
/>
