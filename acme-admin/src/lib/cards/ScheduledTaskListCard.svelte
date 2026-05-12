<script lang="ts">
  import { goto } from "$app/navigation";
  import type { MenuItem } from "@poodle/svelte";
  import { EntityListCard } from "@decodelabs/underlay/templates";
  import type { ScheduledTaskSummary } from "@api-client";

  interface Props {
    task: ScheduledTaskSummary;
    href?: string;
    contextMenuItems?: MenuItem[];
    onContextAction?: (value: string) => void;
  }

  let {
    task,
    href = `/system/scheduled-tasks/${encodeURIComponent(task.id)}`,
    contextMenuItems = [],
    onContextAction
  }: Props = $props();

  function formatTaskName(name: string): string {
    return name
      .split("_")
      .map((word, index) => index === 0 ? word.charAt(0).toUpperCase() + word.slice(1) : word)
      .join(" ");
  }

  function navigate() {
    goto(href);
  }

  function formatLastRun(value: string | null | undefined): string {
    if (!value) {
      return "Never run";
    }

    const then = new Date(value).getTime();
    const now = Date.now();
    const diffMs = Math.max(0, now - then);
    const minute = 60_000;
    const hour = 60 * minute;
    const day = 24 * hour;

    if (diffMs < minute) return "just now";
    if (diffMs < hour) return `${Math.floor(diffMs / minute)}m ago`;
    if (diffMs < day) return `${Math.floor(diffMs / hour)}h ago`;
    return `${Math.floor(diffMs / day)}d ago`;
  }
</script>

<EntityListCard
  title={formatTaskName(task.name)}
  subtitle={task.schedule}
  meta={formatLastRun(task.lastCompletedAt)}
  notLive={!task.enabled}
  leadingIcon="calendar"
  badges={task.enabled ? [] : [{ label: "Disabled", tone: "neutral", appearance: "subtle", size: "sm", muted: true }]}
  contextMenuItems={contextMenuItems}
  contextMenuAriaLabel="Scheduled task actions"
  contextMenuTrigger="leading"
  onContextAction={onContextAction}
  onClick={navigate}
/>
