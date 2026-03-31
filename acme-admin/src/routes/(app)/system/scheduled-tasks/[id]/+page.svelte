<script lang="ts">
import {
  useToasts,
  useAuthenticatedData
} from "@decodelabs/underlay/runtime";
import {
  Callout as PoodleCallout,
  Tabs } from "@poodle/svelte-primitives";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
    import { DataTable, PageHeader as PoodlePageHeader, PageLoading, type TableColumn, type TableRow } from "@poodle/svelte-composites";
  import {
    Card as PoodleCard,
    Code as PoodleCode,
    IconButton as PoodleIconButton,
    MetaBar as PoodleMetaBar,
    MetaItem as PoodleMetaItem,
    Menu as PoodleMenu,
    Pill as PoodlePill,
    TimeAgo
  } from "@poodle/svelte-primitives";
  import type { MenuItem } from "@poodle/svelte-primitives";
  import { adminCommands } from "@api-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import type { ScheduledTaskDetail, JobSummary } from "@api-client";

  const toastStore = useToasts();
  const taskId = $page.params.id;

  let activeTab = $state("details");
  const mountedTabsSet = new Set<string>();
  let mountedTabsVersion = $state(0);

  // Fetch task detail
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      if (!taskId) throw new Error("Task ID is required");
      const task = await adminCommands.getScheduledTask(taskId, fetch, token);
      return { task };
    },
    {
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
      getAuthLoading: () => true,
      defaultValue: { jobs: [] as JobSummary[] }
    }
  );

  // Fetch jobs when task is loaded and tab is job-runs
  $effect(() => {
    if (pageData.data?.task && activeTab === "job-runs") {
      jobsData.tryFetch($authLoading, $currentUser);
    }
  });

  $effect(() => {
    if (activeTab && !mountedTabsSet.has(activeTab)) {
      mountedTabsSet.add(activeTab);
      mountedTabsVersion++;
    }
  });

  function isTabMounted(value: string): boolean {
    void mountedTabsVersion;
    return mountedTabsSet.has(value);
  }

  const task = $derived(pageData.data?.task);
  const jobs = $derived(jobsData.data?.jobs ?? []);
  const jobRows = $derived<TableRow<JobSummary>[]>(
    jobs.map((job) => ({
      id: job.id,
      cells: {
        status: getStatusLabel(job.status),
        attempts: `${job.attempts}/${job.maxAttempts}`,
        createdAt: job.createdAt,
        finishedAt: job.finishedAt ?? "—"
      },
      data: job
    }))
  );

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

  const menuItems = $derived<MenuItem[]>(
    task
      ? [
          { value: "trigger", label: "Trigger now" },
          { value: "toggle", label: task.enabled ? "Disable task" : "Enable task" },
          { value: "separator", label: "", kind: "separator" as const },
          { value: "refresh", label: "Refresh" }
        ]
      : []
  );

  function handleMenuAction(value: string) {
    if (value === "trigger") {
      void handleTrigger();
      return;
    }

    if (value === "toggle") {
      void handleToggle();
      return;
    }

    if (value === "refresh") {
      handleRefresh();
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

  function navigateToJob(job: JobSummary) {
    goto(`/system/jobs/${encodeURIComponent(job.id)}`);
  }

  const jobColumns: TableColumn[] = [
    { id: "status", label: "Status", width: "minmax(100px, 1fr)" },
    { id: "attempts", label: "Attempts", width: "minmax(80px, 1fr)", align: "center" },
    { id: "createdAt", label: "Created", width: "minmax(120px, 1fr)" },
    { id: "finishedAt", label: "Finished", width: "minmax(120px, 1fr)" }
  ];
</script>

{#if pageData.loading && !task}
    <PageLoading presentation="inline" message="Loading task details..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if task}
  <section class="task-detail-page">
    <div class="task-detail-page__header">
      <PoodlePageHeader
        section="Scheduled Task"
        title={formatTaskName(task.name)}
        backHref="/system/scheduled-tasks"
        backLabel="Back to tasks"
      >
        {#snippet actions()}
          <PoodleMenu items={menuItems} ariaLabel="Task actions" placement="bottom-end" on:action={(event) => handleMenuAction(event.detail.value)}>
            <PoodleIconButton slot="trigger" icon="ellipsis" ariaLabel="Task actions" />
          </PoodleMenu>
        {/snippet}
      </PoodlePageHeader>

      <div class="task-detail-page__meta">
      <PoodleMetaBar ariaLabel="Scheduled task metadata">
        <PoodleMetaItem label="ID">
          <PoodleCode inline source={task.id} showCopyButton />
        </PoodleMetaItem>
        <PoodlePill tone={task.enabled ? "success" : "neutral"} appearance="badge" size="lg">
          {task.enabled ? "Enabled" : "Disabled"}
        </PoodlePill>
      </PoodleMetaBar>
      </div>
    </div>

    <Tabs
      bind:value={activeTab}
      items={[
        { value: "details", label: "Details" },
        { value: "job-runs", label: "Job Runs" }
      ]}
      variant="card"
      size="sm"
      ariaLabel="Detail sections"
      let:activeValue
    >
      {#if isTabMounted(activeValue)}
      {#if activeValue === "details"}
        <div class="details-content">
        <div class="task-detail-page__grid">
          <PoodleCard>
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
          </PoodleCard>

          <PoodleCard>
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
          </PoodleCard>
        </div>

        <PoodleCard>
          <div class="task-detail-page__section">
            <h3>Schedule</h3>
            <p class="task-detail-page__schedule-help">
              {describeSchedule(task.schedule)}
            </p>
          </div>
        </PoodleCard>

        <PoodleCard>
          <div class="task-detail-page__section">
            <h3>Payload</h3>
            <pre class="task-detail-page__code">{JSON.stringify(task.payload, null, 2)}</pre>
          </div>
          </PoodleCard>
        </div>
      {:else if activeValue === "job-runs"}
        <div class="jobs-list">
          <DataTable
            rows={jobRows}
            columns={jobColumns}
            loading={jobsData.loading}
            emptyMessage="No job runs found for this task"
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
              {:else}
                {row.cells[column.id] ?? "—"}
              {/if}
            </svelte:fragment>
          </DataTable>
        </div>
      {/if}
      {/if}
    </Tabs>
  </section>
{/if}

<style>
  .task-detail-page {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .task-detail-page__header {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .task-detail-page__meta {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

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
