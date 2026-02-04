<script lang="ts">
  import { adminCommands, type JobSummary, type JobStats, type JobStatus } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import { useAuthenticatedData, PageHeader, useToasts } from "@decodelabs/underlay/patterns";
  import {
    Button,
    PageLoading,
    FormError,
    Badge,
    DataTable,
    StatCard,
    StatGrid,
    TimeAgo,
    Tooltip,
    type DataTableColumn,
    type DataTableAction,
    type DataTableFilters
  } from "@decodelabs/underlay/components";
  import Play from "lucide-svelte/icons/play";
  import Pause from "lucide-svelte/icons/pause";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import CheckCircle from "lucide-svelte/icons/check-circle";
  import XCircle from "lucide-svelte/icons/x-circle";
  import Clock from "lucide-svelte/icons/clock";
  import AlertCircle from "lucide-svelte/icons/alert-circle";

  const toastStore = useToasts();

  // Filter state
  let statusFilter = $state<JobStatus | "">("");
  let jobTypeFilter = $state<string>("");

  // Fetch jobs and stats
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const [jobs, stats] = await Promise.all([
        adminCommands.listJobs(fetch, token, {
          status: statusFilter || undefined,
          jobType: jobTypeFilter || undefined,
          limit: 50
        }),
        adminCommands.getJobStats(fetch, token)
      ]);
      return { jobs, stats };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { jobs: [] as JobSummary[], stats: null as JobStats | null }
    }
  );

  // Trigger fetch when auth is ready
  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  // Refetch when filter changes
  $effect(() => {
    void statusFilter;
    void jobTypeFilter;
    if ($currentUser) {
      pageData.refetch();
    }
  });

  const jobs = $derived(pageData.data?.jobs ?? []);
  const stats = $derived(pageData.data?.stats);

  async function handleRetry(job: JobSummary) {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await adminCommands.retryJob(job.id, fetch, token);
      toastStore.push({ variant: "success", message: "Job retried" });
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to retry job";
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
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to cancel job";
      toastStore.push({ variant: "error", message });
    }
  }

  type BadgeVariant = "default" | "success" | "warning" | "danger" | "info" | "muted";

  function getStatusVariant(status: string): BadgeVariant {
    switch (status) {
      case "succeeded": return "success";
      case "failed": return "danger";
      case "running": return "info";
      case "pending": return "warning";
      case "cancelled": return "muted";
      default: return "default";
    }
  }

  function getStatusLabel(status: string): string {
    return status.charAt(0).toUpperCase() + status.slice(1);
  }

  // Column configuration
  const columns: DataTableColumn<JobSummary>[] = [
    {
      key: "jobType",
      label: "Job Type",
      width: "2fr",
      filterable: true
    },
    {
      key: "status",
      label: "Status",
      width: "100px",
      filterable: true,
      filterType: "select",
      filterOptions: [
        { value: "pending", label: "Pending" },
        { value: "running", label: "Running" },
        { value: "succeeded", label: "Succeeded" },
        { value: "failed", label: "Failed" },
        { value: "cancelled", label: "Cancelled" }
      ]
    },
    {
      key: "attempts",
      label: "Attempts",
      width: "80px",
      align: "center",
      formatter: (_, row) => `${row.attempts}/${row.maxAttempts}`
    },
    {
      key: "createdAt",
      label: "Created",
      width: "100px",
      hideOnMobile: true
    },
    {
      key: "finishedAt",
      label: "Finished",
      width: "100px",
      hideOnMobile: true
    }
  ];

  // Dynamic row actions based on job status
  function getRowActions(job: JobSummary): DataTableAction<JobSummary>[] {
    const actions: DataTableAction<JobSummary>[] = [];

    if (job.status === "failed" || job.status === "cancelled") {
      actions.push({
        label: "Retry",
        onClick: handleRetry
      });
    } else if (job.status === "pending" || job.status === "running") {
      actions.push({
        label: "Cancel",
        onClick: handleCancel,
        variant: "danger"
      });
    }

    return actions;
  }

  function handleFilterChange(filters: DataTableFilters) {
    if (filters.status !== undefined) {
      statusFilter = filters.status as JobStatus | "";
    }
    if (filters.jobType !== undefined) {
      jobTypeFilter = filters.jobType;
    }
  }
</script>

<PageHeader title="Job queue" backHref="/system" backLabel="Back to system">
  {#snippet actions()}
    <Tooltip content="Refresh" inline>
      {#snippet trigger()}
        <Button type="button" variant="subtle" size="icon" onclick={() => pageData.refetch()}>
          <RefreshCw size={16} />
        </Button>
      {/snippet}
    </Tooltip>
  {/snippet}
</PageHeader>

{#if pageData.loading && jobs.length === 0}
  <PageLoading message="Loading jobs..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else}
  <!-- Stats cards -->
  {#if stats}
    <div class="stats-grid">
      <StatGrid columns={4} minItemWidth={180}>
        <StatCard value={stats.pending} label="Pending" variant="warning" compact>
          {#snippet icon()}
            <Clock size={24} />
          {/snippet}
        </StatCard>
        <StatCard value={stats.running} label="Running" variant="info" compact>
          {#snippet icon()}
            <Play size={24} />
          {/snippet}
        </StatCard>
        <StatCard value={stats.failed} label="Failed" variant="danger" compact>
          {#snippet icon()}
            <XCircle size={24} />
          {/snippet}
        </StatCard>
        <StatCard value={stats.succeededRecent} label="Recent Success" variant="success" compact>
          {#snippet icon()}
            <CheckCircle size={24} />
          {/snippet}
        </StatCard>
      </StatGrid>
    </div>
  {/if}

  <DataTable
    data={jobs}
    {columns}
    actions={getRowActions}
    loading={pageData.loading}
    emptyMessage="No jobs found"
    compact
    onFilter={handleFilterChange}
    extendedRowWhen={(row) => !!row.errorMessage}
  >
    {#snippet cell({ column, row, value })}
      {#if column.key === "jobType"}
        <code class="job-type">{value}</code>
      {:else if column.key === "status"}
        <Badge variant={getStatusVariant(row.status)} size="sm">
          {getStatusLabel(row.status)}
        </Badge>
      {:else if column.key === "createdAt"}
        <TimeAgo date={row.createdAt} tooltipFormat="datetime" short />
      {:else if column.key === "finishedAt"}
        {#if row.finishedAt}
          <TimeAgo date={row.finishedAt} tooltipFormat="datetime" short />
        {:else}
          —
        {/if}
      {:else}
        {value}
      {/if}
    {/snippet}
    {#snippet extendedRow({ row })}
      {#if row.errorMessage}
        <div class="error-message">
          <AlertCircle size={14} />
          {row.errorMessage}
        </div>
      {/if}
    {/snippet}
    {#snippet empty()}
      <div class="empty-state">
        <AlertCircle size={32} />
        <p>No jobs found</p>
      </div>
    {/snippet}
  </DataTable>
{/if}

<style>
  .stats-grid {
    margin-bottom: 1.5rem;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 3rem;
    color: var(--admin-color-text-muted);
  }

  .job-type {
    font-family: monospace;
    font-size: 0.8rem;
    background: var(--admin-color-surface-subtle);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    color: var(--admin-color-text);
  }

  .error-message {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    font-size: 0.8rem;
    color: var(--admin-color-danger-text, #fca5a5);
    background: var(--admin-color-danger-subtle, rgba(239, 68, 68, 0.15));
    padding: 0.5rem 0.75rem;
    border-radius: 0.25rem;
    word-break: break-word;
  }

  .error-message :global(svg) {
    flex-shrink: 0;
    margin-top: 0.125rem;
  }
</style>
