<script lang="ts">
  import { adminCommands, type JobSummary, type JobStats, type JobStatus } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import { useAuthenticatedData, PageHeader, useToasts } from "@decodelabs/underlay/patterns";
  import { Button, PageLoading, FormError, Badge, Select, Card } from "@decodelabs/underlay/components";
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

  // Fetch jobs and stats
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const [jobs, stats] = await Promise.all([
        adminCommands.listJobs(fetch, token, {
          status: statusFilter || undefined,
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
    // Track the filter value
    void statusFilter;
    if ($currentUser) {
      pageData.refetch();
    }
  });

  const jobs = $derived(pageData.data?.jobs ?? []);
  const stats = $derived(pageData.data?.stats);

  async function handleRetry(jobId: string) {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await adminCommands.retryJob(jobId, fetch, token);
      toastStore.push({ variant: "success", message: "Job retried" });
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to retry job";
      toastStore.push({ variant: "error", message });
    }
  }

  async function handleCancel(jobId: string) {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await adminCommands.cancelJob(jobId, fetch, token);
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

  function formatRelativeTime(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d ago`;
    if (hours > 0) return `${hours}h ago`;
    if (minutes > 0) return `${minutes}m ago`;
    return "just now";
  }

  const statusOptions = [
    { value: "", label: "All statuses" },
    { value: "pending", label: "Pending" },
    { value: "running", label: "Running" },
    { value: "succeeded", label: "Succeeded" },
    { value: "failed", label: "Failed" },
    { value: "cancelled", label: "Cancelled" }
  ];
</script>

<PageHeader title="Job queue" backHref="/system" backLabel="Back to system">
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
          <span class="stat-icon" style="color: var(--warning, #f59e0b);">
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
          <span class="stat-icon" style="color: var(--info, #3b82f6);">
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
          <span class="stat-icon" style="color: var(--danger, #ef4444);">
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
          <span class="stat-icon" style="color: var(--success, #10b981);">
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

  <!-- Jobs list -->
  <div class="jobs-list">
    {#if jobs.length === 0}
      <div class="empty-state">
        <AlertCircle size={32} />
        <p>No jobs found</p>
      </div>
    {:else}
      <table class="jobs-table">
        <thead>
          <tr>
            <th>Job Type</th>
            <th>Status</th>
            <th>Attempts</th>
            <th>Created</th>
            <th>Finished</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each jobs as job (job.id)}
            <tr>
              <td>
                <code class="job-type">{job.jobType}</code>
              </td>
              <td>
                <Badge variant={getStatusVariant(job.status)} size="sm">
                  {getStatusLabel(job.status)}
                </Badge>
              </td>
              <td class="attempts">
                {job.attempts}/{job.maxAttempts}
              </td>
              <td class="time">
                {formatRelativeTime(job.createdAt)}
              </td>
              <td class="time">
                {#if job.finishedAt}
                  {formatRelativeTime(job.finishedAt)}
                {:else}
                  —
                {/if}
              </td>
              <td class="actions">
                {#if job.status === "failed" || job.status === "cancelled"}
                  <Button
                    type="button"
                    variant="subtle"
                    size="sm"
                    onclick={() => handleRetry(job.id)}
                  >
                    <RefreshCw size={14} />
                    Retry
                  </Button>
                {:else if job.status === "pending" || job.status === "running"}
                  <Button
                    type="button"
                    variant="subtle"
                    size="sm"
                    onclick={() => handleCancel(job.id)}
                  >
                    <Pause size={14} />
                    Cancel
                  </Button>
                {/if}
              </td>
            </tr>
            {#if job.errorMessage}
              <tr class="error-row">
                <td colspan="6">
                  <div class="error-message">
                    <AlertCircle size={14} />
                    {job.errorMessage}
                  </div>
                </td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
    {/if}
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
    border: 1px solid var(--admin-color-border-subtle);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 3rem;
    color: var(--admin-color-text-muted);
  }

  .jobs-table {
    width: 100%;
    border-collapse: collapse;
  }

  .jobs-table th,
  .jobs-table td {
    padding: 0.75rem 1rem;
    text-align: left;
    border-bottom: 1px solid var(--admin-color-border-subtle);
  }

  .jobs-table th {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--admin-color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    background: var(--admin-color-surface-subtle);
  }

  .job-type {
    font-family: monospace;
    font-size: 0.8rem;
    background: var(--admin-color-surface-subtle);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    color: var(--admin-color-text);
  }

  .attempts {
    font-size: 0.875rem;
    color: var(--admin-color-text-muted);
  }

  .time {
    font-size: 0.875rem;
    color: var(--admin-color-text-muted);
  }

  .actions {
    white-space: nowrap;
  }

  .error-row td {
    padding: 0 1rem 0.75rem;
    border-bottom: 1px solid var(--admin-color-border-subtle);
  }

  .error-message {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    font-size: 0.8rem;
    color: #fca5a5;
    background: rgba(239, 68, 68, 0.15);
    padding: 0.5rem 0.75rem;
    border-radius: 0.25rem;
  }

  .error-message :global(svg) {
    flex-shrink: 0;
    margin-top: 0.125rem;
  }
</style>
