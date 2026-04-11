<script lang="ts">
import {
  useToasts
} from "@decodelabs/underlay/runtime/feedback";
import {
  useAuthenticatedData
} from "@decodelabs/underlay/runtime/auth";
import {
  PageHeader as PoodlePageHeader } from "@poodle/svelte";
  import { Callout as PoodleCallout } from "@poodle/svelte";
  import { goto } from "$app/navigation";
  import { adminCommands,
  type JobSummary,
  type JobStats,
  type JobStatus } from "@api-client";
  import { auth } from "$lib/stores/auth";
    import {
    PageLoading,
        DataTable,
    type TableColumn,
    type TableRow
  } from "@poodle/svelte";
  import {
    Button as PoodleButton,
    Card as PoodleCard,
    Field as PoodleField,
    IconButton as PoodleIconButton,
    Menu as PoodleMenu,
    Pill as PoodlePill,
    Select as PoodleSelect,
    TimeAgo
  } from "@poodle/svelte";
  import type { MenuItem } from "@poodle/svelte";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import Clock from "lucide-svelte/icons/clock";
  import Play from "lucide-svelte/icons/play";
  import CheckCircle from "lucide-svelte/icons/check-circle";
  import XCircle from "lucide-svelte/icons/x-circle";
  import AlertCircle from "lucide-svelte/icons/alert-circle";

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
      defaultValue: { jobs: [] as JobSummary[], stats: null as JobStats | null }
    }
  );

  // Track whether initial fetch has completed
  let hasFetched = $state(false);
  $effect(() => {
    if (pageData.data && !pageData.loading) {
      hasFetched = true;
    }
  });

  // Refetch when filter changes (skip first run to avoid double-fetch on mount)
  $effect(() => {
    void statusFilter;
    if (hasFetched) {
      pageData.refetch();
    }
  });

  const jobs = $derived(pageData.data?.jobs ?? []);
  const stats = $derived(pageData.data?.stats);
  const rows = $derived<TableRow<JobSummary>[]>(
    jobs.map((job) => ({
      id: job.id,
      cells: {
        jobType: formatJobType(job.jobType),
        status: getStatusLabel(job.status),
        attempts: `${job.attempts}/${job.maxAttempts}`,
        createdAt: job.createdAt,
        finishedAt: job.finishedAt ?? "—",
        actions: ""
      },
      data: job
    }))
  );
  const expandedRowIds = $derived(
    rows.filter((row) => Boolean((row.data as JobSummary | undefined)?.errorMessage)).map((row) => row.id)
  );

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

  function getStatusTone(status: string): "neutral" | "success" | "danger" {
    switch (status) {
      case "succeeded": return "success";
      case "failed": return "danger";
      default: return "neutral";
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

  function getMenuItems(job: JobSummary): MenuItem[] {
    const items: MenuItem[] = [
      { value: "view", label: "View details" }
    ];

    if (job.status === "failed" || job.status === "cancelled") {
      items.push({ value: "retry", label: "Retry" });
    }

    if (job.status === "pending" || job.status === "running") {
      items.push({ value: "cancel", label: "Cancel" });
    }

    return items;
  }

  function handleMenuAction(job: JobSummary, value: string) {
    if (value === "view") {
      navigateToJob(job);
      return;
    }

    if (value === "retry") {
      void handleRetry(job);
      return;
    }

    if (value === "cancel") {
      void handleCancel(job);
    }
  }

  const statusOptions = [
    { value: "", label: "All statuses" },
    { value: "pending", label: "Pending" },
    { value: "running", label: "Running" },
    { value: "succeeded", label: "Succeeded" },
    { value: "failed", label: "Failed" },
    { value: "cancelled", label: "Cancelled" }
  ];

  const columns: TableColumn[] = [
    { id: "jobType", label: "Job Type", width: "minmax(200px, 2fr)" },
    { id: "status", label: "Status", width: "100px" },
    { id: "attempts", label: "Attempts", width: "80px", align: "center" },
    { id: "createdAt", label: "Created", width: "120px", hideOnMobile: true },
    { id: "finishedAt", label: "Finished", width: "120px", hideOnMobile: true },
    { id: "actions", label: "", width: "60px", align: "center", hideable: false, isRowHeader: false }
  ];
</script>

<PoodlePageHeader title="Job Queue" backHref="/system" backLabel="Back to system">
  {#snippet actions()}
    <PoodleButton type="button" variant="ghost" on:click={() => pageData.refetch()}>
      <RefreshCw size={16} />
      Refresh
    </PoodleButton>
  {/snippet}
</PoodlePageHeader>

{#if pageData.loading && jobs.length === 0}
  <PageLoading presentation="inline" message="Loading jobs..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else}
  <!-- Stats cards -->
  {#if stats}
    <div class="stats-grid">
      <PoodleCard>
        <div class="stat">
          <span class="stat-icon" style="color: var(--admin-color-warning, #f59e0b);">
            <Clock size={24} />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.pending}</span>
            <span class="stat-label">Pending</span>
          </div>
        </div>
      </PoodleCard>
      <PoodleCard>
        <div class="stat">
          <span class="stat-icon" style="color: var(--admin-color-info, #3b82f6);">
            <Play size={24} />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.running}</span>
            <span class="stat-label">Running</span>
          </div>
        </div>
      </PoodleCard>
      <PoodleCard>
        <div class="stat">
          <span class="stat-icon" style="color: var(--admin-color-danger, #ef4444);">
            <XCircle size={24} />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.failed}</span>
            <span class="stat-label">Failed</span>
          </div>
        </div>
      </PoodleCard>
      <PoodleCard>
        <div class="stat">
          <span class="stat-icon" style="color: var(--admin-color-success, #10b981);">
            <CheckCircle size={24} />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.succeededRecent}</span>
            <span class="stat-label">Recent Success</span>
          </div>
        </div>
      </PoodleCard>
    </div>
  {/if}

  <!-- Filter -->
  <div class="filter-bar">
    <PoodleField id="system-jobs-status-filter" label="Status" let:describedBy>
      <PoodleSelect
        id="system-jobs-status-filter"
        value={statusFilter}
        describedBy={describedBy}
        options={statusOptions}
        on:valueChange={(event) => {
          statusFilter = event.detail.value;
        }}
      />
    </PoodleField>
  </div>

  <!-- Jobs table -->
  <div class="jobs-list">
    <DataTable
      {columns}
      {rows}
      {expandedRowIds}
      loading={pageData.loading}
      emptyMessage="No jobs found"
      showLimitSelector={false}
      showRowActions={false}
      on:rowClick={(event) => navigateToJob(event.detail.row.data as JobSummary)}
    >
      <svelte:fragment slot="cell" let:column let:row>
        {@const job = row.data as JobSummary | undefined}
        {#if column.id === "status" && job}
          <PoodlePill tone={getStatusTone(job.status)} appearance="badge" size="lg">
            {getStatusLabel(job.status)}
          </PoodlePill>
        {:else if column.id === "createdAt" && job}
          <TimeAgo datetime={job.createdAt} tooltipFormat="datetime" short />
        {:else if column.id === "finishedAt" && job}
          {#if job.finishedAt}
            <TimeAgo datetime={job.finishedAt} tooltipFormat="datetime" short />
          {:else}
            —
          {/if}
        {:else if column.id === "actions" && job}
          <div
            class="actions-cell"
            role="button"
            tabindex="0"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.stopPropagation()}
          >
            <PoodleMenu items={getMenuItems(job)} ariaLabel="Job actions" placement="bottom-end" on:action={(event) => handleMenuAction(job, event.detail.value)}>
              <PoodleIconButton slot="trigger" icon="ellipsis" ariaLabel="Job actions" />
            </PoodleMenu>
          </div>
        {:else}
          {row.cells[column.id] ?? "—"}
        {/if}
      </svelte:fragment>
      <svelte:fragment slot="expandedRow" let:row>
        {@const job = row.data as JobSummary | undefined}
        {#if job?.errorMessage}
          <div class="error-message">
            <AlertCircle size={14} />
            {job.errorMessage}
          </div>
        {/if}
      </svelte:fragment>
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
