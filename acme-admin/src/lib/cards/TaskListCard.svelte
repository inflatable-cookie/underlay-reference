<script lang="ts">
  import { EntityListCard, type EntityListCardBadge } from "@decodelabs/underlay/templates";
  import type { TaskWithLabels } from "@api-client";

  interface Props {
    task: TaskWithLabels;
    href?: string;
    selectionMode?: boolean;
    reorderMode?: boolean;
    selected?: boolean;
    onSelectionChange?: (selected: boolean) => void;
  }

  let {
    task,
    href = `/projects/${task.projectId}/tasks/${task.id}`,
    selectionMode = false,
    reorderMode = false,
    selected = false,
    onSelectionChange
  }: Props = $props();

  const badges = $derived<EntityListCardBadge[]>([
    {
      label:
        task.status === "completed"
          ? "Done"
          : task.status === "in_progress"
            ? "In Progress"
            : "Pending",
      tone: task.status === "completed" ? "success" : "neutral",
      appearance: "badge",
      size: "sm"
    },
    {
      label: task.priority,
      tone: task.priority === "urgent" ? "danger" : "neutral",
      appearance: "badge",
      size: "sm"
    }
  ]);
</script>

<EntityListCard
  title={task.title}
  href={selectionMode || reorderMode ? undefined : href}
  layout="compact"
  leadingIcon="check-square"
  {selectionMode}
  {reorderMode}
  {selected}
  {badges}
  onSelectionChange={onSelectionChange}
/>
