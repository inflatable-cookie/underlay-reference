<script lang="ts">
  import type { QueryParams } from "@decodelabs/underlay/client/query";
  import { SystemJobListPage, toPagedListResult } from "@decodelabs/underlay/templates";
  import type { SystemJobListRequest } from "@decodelabs/underlay/templates";
  import {
    adminCommands,
    type JobStatus,
    type JobSummary
  } from "@api-client";

  interface Props {
    title?: string;
    backHref?: string;
    backLabel?: string;
    query: QueryParams;
    onQueryChange: (query: QueryParams) => void;
  }

  let {
    title = "Job Queue",
    backHref = "/system",
    backLabel = "Back to system",
    query,
    onQueryChange
  }: Props = $props();

  function getStatus(status: SystemJobListRequest["status"]): JobStatus | undefined {
    return typeof status === "string" ? status as JobStatus : undefined;
  }
</script>

<SystemJobListPage
  {title}
  {backHref}
  {backLabel}
  {query}
  {onQueryChange}
  dataLoader={async (fetch, token, request) => {
    const response = await adminCommands.listJobs(fetch, token, {
      status: getStatus(request.status),
      page: request.page,
      limit: request.limit
    });
    return toPagedListResult<JobSummary>(response);
  }}
  statsLoader={async (fetch, token) => await adminCommands.getJobStats(fetch, token)}
  retryAction={async (job, fetch, token) => {
    await adminCommands.retryJob(job.id, fetch, token);
  }}
  cancelAction={async (job, fetch, token) => {
    await adminCommands.cancelJob(job.id, fetch, token);
  }}
/>
