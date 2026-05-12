<script lang="ts">
  import { goto } from "$app/navigation";
  import type { QueryParams } from "@decodelabs/underlay/client/query";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";
  import { EntityListPage, toPagedListResult } from "@decodelabs/underlay/templates";
  import { adminCommands, type ScheduledTaskSummary } from "@api-client";
  import type { MenuItem } from "@poodle/svelte";
  import { ScheduledTaskListCard } from "$lib/cards";
  import { auth } from "$lib/stores/auth";

  interface Props {
    title?: string;
    backHref?: string;
    backLabel?: string;
    query: QueryParams;
    onQueryChange: (query: QueryParams) => void;
  }

  let {
    title = "Scheduled Tasks",
    backHref = "/system",
    backLabel = "Back to system",
    query,
    onQueryChange
  }: Props = $props();

  const toastStore = useToasts();
  let refreshRevision = $state(0);

  const filters = [
    {
      id: "enabled",
      type: "select" as const,
      label: "Status",
      options: [
        { value: "All", label: "All tasks" },
        { value: "true", label: "Enabled only" },
        { value: "false", label: "Disabled only" }
      ]
    }
  ];

  function getEnabledFilter(nextQuery: QueryParams): boolean | undefined {
    const filter = nextQuery.filters?.find((entry) => entry.field === "enabled");
    if (!filter || filter.value === "" || filter.value === "All") return undefined;
    return filter.value === "true" ? true : filter.value === "false" ? false : undefined;
  }

  async function dataLoader(fetch: typeof window.fetch, token: string | null, nextQuery: QueryParams) {
    if (!token) throw new Error("Not authenticated");
    void refreshRevision;

    const response = await adminCommands.listScheduledTasks(fetch, token, {
      enabled: getEnabledFilter(nextQuery),
      page: nextQuery.page ?? 1,
      limit: nextQuery.limit ?? 30
    });
    return toPagedListResult(response);
  }

  async function handleToggle(task: ScheduledTaskSummary) {
    const token = auth.getToken();
    if (!token) return;

    try {
      await adminCommands.toggleScheduledTask(task.id, !task.enabled, fetch, token);
      toastStore.push({
        variant: "success",
        message: task.enabled ? "Task disabled" : "Task enabled"
      });
      refreshRevision += 1;
    } catch {
      toastStore.push({ variant: "error", message: "Failed to toggle task" });
    }
  }

  async function handleTrigger(task: ScheduledTaskSummary) {
    const token = auth.getToken();
    if (!token) return;

    try {
      const result = await adminCommands.triggerScheduledTask(task.id, fetch, token);
      toastStore.push({ variant: "success", message: "Job created" });
      goto(`/system/jobs/${result.jobId}`);
    } catch {
      toastStore.push({ variant: "error", message: "Failed to trigger task" });
    }
  }

  function getMenuItems(task: ScheduledTaskSummary): MenuItem[] {
    return [
      { value: "trigger", label: "Trigger now" },
      { value: "toggle", label: task.enabled ? "Disable task" : "Enable task" }
    ];
  }

  function handleMenuAction(task: ScheduledTaskSummary, value: string) {
    if (value === "trigger") {
      void handleTrigger(task);
      return;
    }

    if (value === "toggle") {
      void handleToggle(task);
    }
  }
</script>

{#snippet renderItem(task: ScheduledTaskSummary)}
  <ScheduledTaskListCard
    {task}
    contextMenuItems={getMenuItems(task)}
    onContextAction={(value) => handleMenuAction(task, value)}
  />
{/snippet}

{#key refreshRevision}
  <EntityListPage
    {title}
    {backHref}
    {backLabel}
    {dataLoader}
    presentation="cards"
    {renderItem}
    {filters}
    {query}
    {onQueryChange}
  />
{/key}
