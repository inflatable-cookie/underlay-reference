<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { auth } from "$lib/stores/auth";
  import {
    buildQueryString,
    parseQueryParams,
    type QueryParams
  } from "@decodelabs/underlay/client/query";
  import { useAuthenticatedData } from "@decodelabs/underlay/runtime/auth";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";
  import { EntityListPage } from "@decodelabs/underlay/templates";
  import {
    Card as PoodleCard,
    Icon,
    Pill as PoodlePill,
    TimeAgo,
    type TableCellValue,
    type TableColumn,
    type TableRow,
    type TableRowAction
  } from "@poodle/svelte";
  import {
    adminCommands,
    type JobStats,
    type JobStatus,
    type JobSummary
  } from "@api-client";

  const toastStore = useToasts();
  const currentQuery = $derived(parseQueryParams($page.url.searchParams));

  let refreshRevision = $state(0);

  const statsData = useAuthenticatedData(
    async (fetch, token) => {
      if (!token) throw new Error("Not authenticated");
      void refreshRevision;
      return await adminCommands.getJobStats(fetch, token);
    },
    {
      defaultValue: null as JobStats | null,
      queryKey: () => `jobs-stats:${refreshRevision}`
    }
  );

  const stats = $derived(statsData.data);

  const columns: TableColumn[] = [
    { id: "jobType", label: "Job Type", width: "minmax(18rem, 2fr)" },
    { id: "status", label: "Status", width: "8rem" },
    { id: "attempts", label: "Attempts", width: "6rem", align: "center" },
    { id: "createdAt", label: "Created", width: "8rem", hideOnMobile: true },
    { id: "finishedAt", label: "Finished", width: "8rem", hideOnMobile: true }
  ];

  const filters = [
    {
      id: "status",
      type: "select" as const,
      label: "Status",
      options: [
        { value: "All", label: "All statuses" },
        { value: "pending", label: "Pending" },
        { value: "claimed", label: "Claimed" },
        { value: "running", label: "Running" },
        { value: "succeeded", label: "Succeeded" },
        { value: "failed", label: "Failed" },
        { value: "cancelled", label: "Cancelled" }
      ]
    }
  ];

  function updateUrl(nextQuery: QueryParams) {
    const url = new URL($page.url);
    url.search = buildQueryString(nextQuery);
    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  function getStatusFilter(query: QueryParams): JobStatus | undefined {
    const filter = query.filters?.find((entry) => entry.field === "status");
    if (!filter || filter.value === "" || filter.value === "All") {
      return undefined;
    }
    return filter.value as JobStatus;
  }

  async function dataLoader(fetch: typeof window.fetch, token: string | null, query: QueryParams) {
    if (!token) throw new Error("Not authenticated");

    return await adminCommands.listJobs(fetch, token, {
      status: getStatusFilter(query),
      page: query.page ?? 1,
      limit: query.limit ?? 30
    });
  }

  function formatJobType(jobType: string): string {
    return jobType
      .replace(/_/g, " ")
      .replace(/\b\w/g, (char) => char.toUpperCase());
  }

  function getStatusTone(status: string): "neutral" | "success" | "danger" {
    switch (status) {
      case "succeeded":
        return "success";
      case "failed":
      case "cancelled":
        return "danger";
      default:
        return "neutral";
    }
  }

  function getStatusLabel(status: string): string {
    return status.charAt(0).toUpperCase() + status.slice(1);
  }

  function navigateToJob(job: JobSummary) {
    goto(`/system/jobs/${encodeURIComponent(job.id)}`);
  }

  async function handleRetry(job: JobSummary) {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await adminCommands.retryJob(job.id, fetch, token);
      toastStore.push({ variant: "success", message: "Job queued for retry" });
      refreshRevision += 1;
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to retry job";
      toastStore.push({ variant: "error", message });
    }
  }

  async function handleCancel(job: JobSummary) {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await adminCommands.cancelJob(job.id, fetch, token);
      toastStore.push({ variant: "success", message: "Job cancelled" });
      refreshRevision += 1;
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to cancel job";
      toastStore.push({ variant: "error", message });
    }
  }

  function rowActions(row: TableRow<JobSummary>) {
    const job = row.data;
    if (!job) return [];

    const actions: { value: string; label: string }[] = [
      { value: "view", label: "View details" }
    ];

    if (job.status === "failed" || job.status === "cancelled") {
      actions.push({ value: "retry", label: "Retry" });
    }

    if (job.status === "pending" || job.status === "claimed" || job.status === "running") {
      actions.push({ value: "cancel", label: "Cancel" });
    }

    return actions;
  }

  function handleRowActionSelect(row: TableRow<JobSummary>, action: TableRowAction) {
    const job = row.data;
    if (!job) return;

    if (action.value === "view") {
      navigateToJob(job);
      return;
    }

    if (action.value === "retry") {
      void handleRetry(job);
      return;
    }

    if (action.value === "cancel") {
      void handleCancel(job);
    }
  }
</script>

{#snippet beforeList()}
  {#if stats}
    <div class="stats-grid">
      <PoodleCard>
        <div class="stat">
          <span class="stat-icon stat-icon--warning">
            <Icon icon="clock-3" size="lg" />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.pending}</span>
            <span class="stat-label">Pending</span>
          </div>
        </div>
      </PoodleCard>
      <PoodleCard>
        <div class="stat">
          <span class="stat-icon stat-icon--info">
            <Icon icon="play" size="lg" />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.running}</span>
            <span class="stat-label">Running</span>
          </div>
        </div>
      </PoodleCard>
      <PoodleCard>
        <div class="stat">
          <span class="stat-icon stat-icon--danger">
            <Icon icon="circle-x" size="lg" />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.failed}</span>
            <span class="stat-label">Failed</span>
          </div>
        </div>
      </PoodleCard>
      <PoodleCard>
        <div class="stat">
          <span class="stat-icon stat-icon--success">
            <Icon icon="circle-check-big" size="lg" />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.succeededRecent}</span>
            <span class="stat-label">Recent Success</span>
          </div>
        </div>
      </PoodleCard>
    </div>
  {/if}
{/snippet}

{#snippet renderCell(column: TableColumn, row: TableRow<JobSummary>, value: TableCellValue)}
  {@const job = row.data}
  {#if !job}
    —
  {:else if column.id === "jobType"}
    <div class="job-cell">
      <span class="job-title">{formatJobType(job.jobType)}</span>
      {#if job.errorMessage}
        <span class="job-error">{job.errorMessage}</span>
      {/if}
    </div>
  {:else if column.id === "status"}
    <PoodlePill tone={getStatusTone(job.status)} appearance="badge" size="sm">
      {getStatusLabel(job.status)}
    </PoodlePill>
  {:else if column.id === "attempts"}
    {job.attempts}/{job.maxAttempts}
  {:else if column.id === "createdAt"}
    <TimeAgo datetime={job.createdAt} tooltipFormat="datetime" short />
  {:else if column.id === "finishedAt"}
    {#if job.finishedAt}
      <TimeAgo datetime={job.finishedAt} tooltipFormat="datetime" short />
    {:else}
      —
    {/if}
  {:else}
    {value ?? "—"}
  {/if}
{/snippet}

{#key refreshRevision}
  <EntityListPage
    title="Job Queue"
    backHref="/system"
    backLabel="Back to system"
    {dataLoader}
    presentation="table"
    {columns}
    {rowActions}
    {renderCell}
    onRowActionSelect={handleRowActionSelect}
    {filters}
    query={currentQuery}
    onQueryChange={updateUrl}
    beforeList={beforeList}
  />
{/key}

<style>
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr));
    gap: 1rem;
  }

  .stat {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem;
  }

  .stat-icon {
    display: flex;
    flex-shrink: 0;
  }

  .stat-icon--warning {
    color: var(--admin-color-warning, #f59e0b);
  }

  .stat-icon--info {
    color: var(--admin-color-info, #3b82f6);
  }

  .stat-icon--danger {
    color: var(--admin-color-danger, #ef4444);
  }

  .stat-icon--success {
    color: var(--admin-color-success, #10b981);
  }

  .stat-content {
    display: flex;
    flex-direction: column;
  }

  .stat-value {
    color: var(--poodle-color-text);
    font-size: 1.5rem;
    font-weight: 600;
    line-height: 1;
  }

  .stat-label {
    color: var(--poodle-color-text-muted);
    font-size: 0.75rem;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .job-cell {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
  }

  .job-title {
    color: var(--poodle-color-text);
    font-weight: 600;
  }

  .job-error {
    color: var(--poodle-color-danger-muted);
    font-size: 0.8rem;
    line-height: 1.35;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
