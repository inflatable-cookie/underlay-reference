<script lang="ts">
  import type { QueryParams } from "@inflatable-cookie/underlay/client/query";
  import {
    SystemScheduledTasksListPage,
    toPagedListResult
  } from "@inflatable-cookie/underlay/templates";
  import { adminCommands, type ScheduledTaskSummary } from "@api-client";

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
</script>

<SystemScheduledTasksListPage
  {title}
  {backHref}
  {backLabel}
  {query}
  {onQueryChange}
  dataLoader={async (fetch, token, request) => {
    const response = await adminCommands.listScheduledTasks(fetch, token, {
      enabled: request.enabled,
      page: request.page,
      limit: request.limit
    });
    return toPagedListResult<ScheduledTaskSummary>(response);
  }}
  triggerAction={async (task, fetch, token) => {
    const result = await adminCommands.triggerScheduledTask(task.id, fetch, token);
    return { jobId: result.jobId };
  }}
  toggleAction={async (task, fetch, token) => {
    await adminCommands.toggleScheduledTask(task.id, !task.enabled, fetch, token);
  }}
/>
