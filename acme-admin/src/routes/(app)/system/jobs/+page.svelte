<script lang="ts">
  import { goto } from "$app/navigation";
  import { adminCommands, type JobSummary, type JobStats, type JobStatus } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import { useAuthenticatedData, PageHeader, useToasts } from "@decodelabs/underlay/patterns";
  import {
    Button,
    Card,
    PageLoading,
    FormError,
    Badge,
    Select,
    DataTable,
    DropdownMenu,
    TimeAgo,
    type DataTableColumn
  } from "@decodelabs/underlay/components";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import Clock from "lucide-svelte/icons/clock";
  import Play from "lucide-svelte/icons/play";
  import CheckCircle from "lucide-svelte/icons/check-circle";
  import XCircle from "lucide-svelte/icons/x-circle";
  import AlertCircle from "lucide-svelte/icons/alert-circle";
  import MoreHorizontal from "lucide-svelte/icons/more-horizontal";

  const toastStore = useToasts();

  // Filter state
  let statusFilter = $state<string>("");

  // Fetch jobs and stats
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const [jobs, stats] = await Promise.all([
        adminCommands.listJobs(fetch, token, {
          status: (statusFilter || undefined) as JobStatus | undefined,
          limit: 100
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
    if ($currentUser) {
      pageData.refetch();
    }
  });

  const jobs = $derived(pageData.data?.jobs ?? []);
  const stats = $derived(pageData.data?.stats);

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

  function formatJobType(jobType: string): string {
    return jobType
      .replace(/_/g, " ")
      .replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function getMenuItems(job: JobSummary) {
    const items: Array<{ label: string; onSelect: () => void; destructive?: boolean }> = [
      { label: "View details", onSelect: () => navigateToJob(job) }
    ];

    if (job.status === "failed" || job.status === "cancelled") {
      items.push({ label: "Retry", onSelect: () => handleRetry(job) });
    }

    if (job.status === "pending" || job.status === "running") {
      items.push({ label: "Cancel", onSelect: () => handleCancel(job), destructive: true });
    }

    return items;
  }

  const statusOptions = [
    { value: "", label: "All statuses" },
    { value: "pending", label: "Pending" },
    { value: "running", label: "Running" },
    { value: "succeeded", label: "Succeeded" },
    { value: "failed", label: "Failed" },
    { value: "cancelled", label: "Cancelled" }
  ];

  const columns: DataTableColumn<JobSummary>[] = [
    { key: "jobType", label: "Job Type", width: "minmax(200px, 2fr)" },
    { key: "status", label: "Status", width: "100px" },
    { key: "attempts", label: "Attempts", width: "80px", align: "center" },
    { key: "createdAt", label: "Created", width: "120px", hideOnMobile: true },
    { key: "finishedAt", label: "Finished", width: "120px", hideOnMobile: true },
    { key: "actions", label: "", width: "60px", align: "center", hideable: false }
  ];
</script>

<PageHeader section="Job Queue" backHref="/system" backLabel="Back to system">
  {#snippet actions()}
    <Button type="button" variant="subtle" onclick={() => pageData.refetch()}>
      <RefreshCw size={16} />
      Refresh
    </Button>
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
      <Card>
        <div class="stat">
          <span class="stat-icon" style="color: var(--admin-color-warning, #f59e0b);">
            <Clock size={24} />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.pending}</span>
            <span class="stat-label">Pending</span>
          </div>
        </div>
      </Card>
      <Card>
        <div class="stat">
          <span class="stat-icon" style="color: var(--admin-color-info, #3b82f6);">
            <Play size={24} />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.running}</span>
            <span class="stat-label">Running</span>
          </div>
        </div>
      </Card>
      <Card>
        <div class="stat">
          <span class="stat-icon" style="color: var(--admin-color-danger, #ef4444);">
            <XCircle size={24} />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.failed}</span>
            <span class="stat-label">Failed</span>
          </div>
        </div>
      </Card>
      <Card>
        <div class="stat">
          <span class="stat-icon" style="color: var(--admin-color-success, #10b981);">
            <CheckCircle size={24} />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.succeededRecent}</span>
            <span class="stat-label">Recent Success</span>
          </div>
        </div>
      </Card>
    </div>
  {/if}

  <!-- Filter -->
  <div class="filter-bar">
    <Select
      bind:value={statusFilter}
      items={statusOptions}
      placeholder="All statuses"
    />
  </div>

  <!-- Jobs table -->
  <div class="jobs-list">
    <DataTable
      data={jobs}
      {columns}
      loading={pageData.loading}
      emptyMessage="No jobs found"
      showLimitSelector={false}
      onRowClick={navigateToJob}
      extendedRowWhen={(row) => !!row.errorMessage}
    >
      {#snippet cell({ column, row })}
        {#if column.key === "jobType"}
          {formatJobType(row.jobType)}
        {:else if column.key === "status"}
          <Badge variant={getStatusVariant(row.status)} size="sm">
            {getStatusLabel(row.status)}
          </Badge>
        {:else if column.key === "attempts"}
          {row.attempts}/{row.maxAttempts}
        {:else if column.key === "createdAt"}
          <TimeAgo date={row.createdAt} tooltipFormat="datetime" short />
        {:else if column.key === "finishedAt"}
          {#if row.finishedAt}
            <TimeAgo date={row.finishedAt} tooltipFormat="datetime" short />
          {:else}
            —
          {/if}
        {:else if column.key === "actions"}
          <div
            class="actions-cell"
            role="button"
            tabindex="0"
            aria-label="Job actions"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.stopPropagation()}
          >
            <DropdownMenu items={getMenuItems(row)} triggerAriaLabel="Job actions">
              {#snippet trigger()}
                <MoreHorizontal size={16} />
              {/snippet}
            </DropdownMenu>
          </div>
        {:else}
          —
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
    </DataTable>
  </div>
{/if}

<style>
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .stat {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem;
  }

  .stat-icon {
    flex-shrink: 0;
    display: flex;
  }

  .stat-content {
    display: flex;
    flex-direction: column;
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--admin-color-text);
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--admin-color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .filter-bar {
    margin-bottom: 1rem;
    max-width: 200px;
  }

  .jobs-list {
    background: var(--admin-color-surface-card);
    border-radius: 0.5rem;
  }

  .jobs-list :global(.underlay-data-table) {
    --underlay-table-border: 1px solid var(--admin-color-border-subtle);
    --underlay-table-header-bg: var(--admin-color-surface-subtle);
    --underlay-table-row-hover: var(--admin-color-surface-subtle);
    --underlay-table-row-selected: var(--admin-color-surface-subtle);
    color: var(--admin-color-text);
  }

  .jobs-list :global(.underlay-data-table-wrapper) {
    border-radius: 0.5rem;
  }

  .jobs-list :global(.underlay-table-cell) {
    padding: 0.75rem 1rem;
  }

  .jobs-list :global(.underlay-table-body > .underlay-table-row) {
    cursor: pointer;
  }

  .jobs-list :global(.underlay-table-body > .underlay-table-row.underlay-has-extended > .underlay-table-cell) {
    background: var(--admin-color-surface-subtle);
  }

  .jobs-list :global(.underlay-table-row--extended > .underlay-table-cell) {
    padding: 0;
    background: var(--admin-color-surface-subtle);
  }

  .actions-cell {
    display: flex;
    justify-content: center;
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
