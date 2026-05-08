<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { auth } from "$lib/stores/auth";
  import { ScheduledTaskListCard } from "$lib/cards";
  import {
    buildQueryString,
    parseQueryParams,
    type QueryParams
  } from "@decodelabs/underlay/client/query";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";
  import { EntityListPage } from "@decodelabs/underlay/templates";
  import { adminCommands, type ScheduledTaskSummary } from "@api-client";
  import type { MenuItem } from "@poodle/svelte";

  const toastStore = useToasts();
  const currentQuery = $derived(parseQueryParams($page.url.searchParams));
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

  function updateUrl(nextQuery: QueryParams) {
    const url = new URL($page.url);
    url.search = buildQueryString(nextQuery);
    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  function getEnabledFilter(query: QueryParams): boolean | undefined {
    const filter = query.filters?.find((entry) => entry.field === "enabled");
    if (!filter || filter.value === "" || filter.value === "All") {
      return undefined;
    }
    return filter.value === "true" ? true : filter.value === "false" ? false : undefined;
  }

  async function dataLoader(fetch: typeof window.fetch, token: string | null, query: QueryParams) {
    if (!token) throw new Error("Not authenticated");
    void refreshRevision;

    return await adminCommands.listScheduledTasks(fetch, token, {
      enabled: getEnabledFilter(query),
      page: query.page ?? 1,
      limit: query.limit ?? 30
    });
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
    title="Scheduled Tasks"
    backHref="/system"
    backLabel="Back to system"
    {dataLoader}
    presentation="cards"
    {renderItem}
    {filters}
    query={currentQuery}
    onQueryChange={updateUrl}
  />
{/key}
