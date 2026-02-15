<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import {
    DetailPageShell,
    DetailMeta,
    DetailMetaId,
    DetailMetaSeparator,
    useToasts,
    useAuthenticatedData
  } from "@decodelabs/underlay/patterns";
  import {
    Badge,
    Card,
    DataTable,
    DropdownMenu,
    FormError,
    PageLoading,
    Pill,
    TimeAgo,
    type DataTableColumn
  } from "@decodelabs/underlay/components";
  import { adminCommands } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import type { ScheduledTaskDetail, JobSummary } from "acme-client";

  const toastStore = useToasts();
  const taskId = $page.params.id;

  let activeTab = $state("details");

  // Fetch task detail
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      if (!taskId) throw new Error("Task ID is required");
      const task = await adminCommands.getScheduledTask(taskId, fetch, token);
      return { task };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { task: undefined as ScheduledTaskDetail | undefined }
    }
  );

  // Fetch jobs for this task's jobType
  const jobsData = useAuthenticatedData(
    async (fetch, token) => {
      const task = pageData.data?.task;
      if (!task) return { jobs: [] };
      const jobs = await adminCommands.listJobs(fetch, token, {
        jobType: task.jobType,
        limit: 50
      });
      return { jobs };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { jobs: [] as JobSummary[] }
    }
  );

  // Trigger fetch when auth is ready
  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  // Fetch jobs when task is loaded and tab is job-runs
  $effect(() => {
    if (pageData.data?.task && activeTab === "job-runs") {
      jobsData.tryFetch($authLoading, $currentUser);
    }
  });

  const task = $derived(pageData.data?.task);
  const jobs = $derived(jobsData.data?.jobs ?? []);

  function formatDate(dateStr: string | null | undefined): string {
    if (!dateStr) return "Never";
    return new Date(dateStr).toLocaleString();
  }

  function formatTaskName(name: string): string {
    return name
      .replace(/_/g, " ")
      .replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function formatJobType(jobType: string): string {
    return jobType
      .replace(/_/g, " ")
      .replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function describeSchedule(schedule: string): string {
    if (schedule === "0 */15 * * * *") return "Every 15 minutes";
    if (schedule === "0 0 * * * *") return "Every hour at :00";
    if (schedule === "0 */5 * * * *") return "Every 5 minutes";
    if (schedule.match(/^0 \d+ \* \* \* \*$/)) {
      const minute = schedule.split(" ")[1];
      return `Every hour at :${minute}`;
    }
    if (schedule.match(/^0 0 \d+ \* \* \*$/)) {
      const hour = schedule.split(" ")[2];
      return `Daily at ${hour}:00`;
    }
    if (schedule.includes("* * 0")) return "Weekly on Sunday";
    return `Cron: ${schedule}`;
  }

  async function handleToggle() {
    if (!task) return;
    const token = auth.getToken();
    if (!token) return;

    try {
      await adminCommands.toggleScheduledTask(task.id, !task.enabled, fetch, token);
      toastStore.push({ variant: "success", message: task.enabled ? "Task disabled" : "Task enabled" });
      pageData.refetch();
    } catch (err) {
      toastStore.push({ variant: "error", message: "Failed to toggle task" });
    }
  }

  async function handleTrigger() {
    if (!task) return;
    const token = auth.getToken();
    if (!token) return;

    try {
      const result = await adminCommands.triggerScheduledTask(task.id, fetch, token);
      toastStore.push({ variant: "success", message: "Job created" });
      goto(`/system/jobs/${result.jobId}`);
    } catch (err) {
      toastStore.push({ variant: "error", message: "Failed to trigger task" });
    }
  }

  function handleRefresh() {
    pageData.refetch();
    if (activeTab === "job-runs") {
      jobsData.refetch();
    }
  }

  const menuItems = $derived(task ? [
    { label: "Trigger now", onSelect: handleTrigger },
    { label: task.enabled ? "Disable task" : "Enable task", onSelect: handleToggle },
    { separator: true },
    { label: "Refresh", onSelect: handleRefresh }
  ] : []);

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

  function navigateToJob(job: JobSummary) {
    goto(`/system/jobs/${encodeURIComponent(job.id)}`);
  }

  const jobColumns: DataTableColumn<JobSummary>[] = [
    { key: "status", label: "Status", width: "minmax(100px, 1fr)" },
    { key: "attempts", label: "Attempts", width: "minmax(80px, 1fr)", align: "center" },
    { key: "createdAt", label: "Created", width: "minmax(120px, 1fr)" },
    { key: "finishedAt", label: "Finished", width: "minmax(120px, 1fr)" }
  ];
</script>

{#if pageData.loading && !task}
  <PageLoading message="Loading task details..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if task}
  <DetailPageShell
    section="Scheduled Task"
    title={formatTaskName(task.name)}
    backHref="/system/scheduled-tasks"
    backLabel="Back to tasks"
    tabs={[
      { value: "details", label: "Details" },
      { value: "job-runs", label: "Job Runs" }
    ]}
    bind:activeTab
  >
    {#snippet meta()}
      <DetailMeta>
        <DetailMetaId value={task.id} />
        <DetailMetaSeparator />
        <Pill accent={task.enabled ? "#10b981" : "#6b7280"}>
          {task.enabled ? "Enabled" : "Disabled"}
        </Pill>
      </DetailMeta>
    {/snippet}

    {#snippet actions()}
      <DropdownMenu items={menuItems} triggerAriaLabel="Task actions" />
    {/snippet}

    {#snippet tabContent(tab)}
      {#if tab === "details"}
        <div class="details-content">
        <div class="task-detail-page__grid">
          <Card>
            <div class="task-detail-page__section">
              <h3>Configuration</h3>
              <dl class="task-detail-page__dl">
                <dt>ID</dt>
                <dd><code>{task.id}</code></dd>
                <dt>Name</dt>
                <dd>{formatTaskName(task.name)}</dd>
                <dt>Job Type</dt>
                <dd>{formatJobType(task.jobType)}</dd>
                <dt>Schedule</dt>
                <dd><code>{task.schedule}</code></dd>
                <dt>Priority</dt>
                <dd>{task.priority}</dd>
                <dt>Max Attempts</dt>
                <dd>{task.maxAttempts}</dd>
                <dt>Timeout</dt>
                <dd>{task.timeoutSeconds ? `${task.timeoutSeconds}s` : "None"}</dd>
                <dt>Allow Overlap</dt>
                <dd>{task.allowOverlap ? "Yes" : "No"}</dd>
              </dl>
            </div>
          </Card>

          <Card>
            <div class="task-detail-page__section">
              <h3>Execution History</h3>
              <dl class="task-detail-page__dl">
                <dt>Last Scheduled</dt>
                <dd>{formatDate(task.lastScheduledAt)}</dd>
                <dt>Last Completed</dt>
                <dd>{formatDate(task.lastCompletedAt)}</dd>
                <dt>Created</dt>
                <dd>{formatDate(task.createdAt)}</dd>
                <dt>Updated</dt>
                <dd>{formatDate(task.updatedAt)}</dd>
              </dl>
            </div>
          </Card>
        </div>

        <Card>
          <div class="task-detail-page__section">
            <h3>Schedule</h3>
            <p class="task-detail-page__schedule-help">
              {describeSchedule(task.schedule)}
            </p>
          </div>
        </Card>

        <Card>
          <div class="task-detail-page__section">
            <h3>Payload</h3>
            <pre class="task-detail-page__code">{JSON.stringify(task.payload, null, 2)}</pre>
          </div>
        </Card>
        </div>
      {:else if tab === "job-runs"}
        <div class="jobs-list">
          <DataTable
            data={jobs}
            columns={jobColumns}
            loading={jobsData.loading}
            emptyMessage="No job runs found for this task"
            showLimitSelector={false}
            onRowClick={navigateToJob}
          >
            {#snippet cell({ column, row })}
              {#if column.key === "status"}
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
              {:else}
                —
              {/if}
            {/snippet}
          </DataTable>
        </div>
      {/if}
    {/snippet}
  </DetailPageShell>
{/if}

<style>
  .details-content {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .task-detail-page__grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1.5rem;
  }

  .task-detail-page__section {
    padding: 0.5rem;
  }

  .task-detail-page__section h3 {
    margin: 0 0 0.75rem 0;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--admin-color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .task-detail-page__dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.375rem 1rem;
    margin: 0;
    font-size: 0.875rem;
  }

  .task-detail-page__dl dt {
    color: var(--admin-color-text-muted);
  }

  .task-detail-page__dl dd {
    margin: 0;
  }

  .task-detail-page__dl code {
    font-size: 0.8rem;
    word-break: break-all;
    background: var(--admin-color-surface-subtle);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }

  .task-detail-page__schedule-help {
    margin: 0;
    font-size: 0.95rem;
    color: var(--admin-color-text);
  }

  .task-detail-page__code {
    margin: 0;
    padding: 0.75rem;
    background: var(--admin-color-surface-subtle);
    border-radius: 0.35rem;
    font-size: 0.8rem;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .jobs-list {
    background: var(--admin-color-surface-card);
    border-radius: 0.5rem;
  }

  .jobs-list :global(.underlay-data-table) {
    --underlay-table-border: 1px solid var(--admin-color-border-subtle);
    --underlay-table-header-bg: var(--admin-color-surface-subtle);
    --underlay-table-row-hover: var(--admin-color-surface-subtle);
    color: var(--admin-color-text);
  }

  .jobs-list :global(.underlay-data-table-wrapper) {
    border-radius: 0.5rem;
  }

  .jobs-list :global(.underlay-table-body > .underlay-table-row) {
    cursor: pointer;
  }
</style>
