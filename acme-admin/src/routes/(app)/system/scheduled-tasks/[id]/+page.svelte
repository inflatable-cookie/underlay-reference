<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import { EntityDetailPage } from "@decodelabs/underlay/templates";
  import { useAuthenticatedData } from "@decodelabs/underlay/runtime/auth";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";
  import {
    Card as PoodleCard,
    Code as PoodleCode,
    DataTable,
    IconButton as PoodleIconButton,
    Menu as PoodleMenu,
    Pill as PoodlePill,
    TimeAgo,
    formatDisplayDateTime,
    type MenuItem,
    type TableColumn,
    type TableRow
  } from "@poodle/svelte";
  import { adminCommands, type JobSummary, type ScheduledTaskDetail } from "@api-client";

  interface Props {
    data: { id: string };
  }

  let { data }: Props = $props();

  const toastStore = useToasts();
  let task = $state<ScheduledTaskDetail | null>(null);
  let activeTab = $state("details");
  let reloadRevision = $state(0);

  async function taskLoader(fetch: typeof window.fetch, token: string | null) {
    if (!token) throw new Error("Not authenticated");
    const result = await adminCommands.getScheduledTask(data.id, fetch, token);
    task = result;
    return result;
  }

  const jobsData = useAuthenticatedData(
    async (fetch, token) => {
      if (!task) return { jobs: [] as JobSummary[] };
      const jobs = await adminCommands.listJobs(fetch, token, {
        jobType: task.jobType,
        limit: 50
      });
      return { jobs: jobs.data };
    },
    {
      getAuthLoading: () => true,
      defaultValue: { jobs: [] as JobSummary[] }
    }
  );

  $effect(() => {
    if (task && activeTab === "job-runs") {
      jobsData.tryFetch($authLoading, $currentUser);
    }
  });

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

  function formatTaskName(name: string): string {
    return name.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function formatJobType(jobType: string): string {
    return jobType.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
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
    const token = auth.getToken();
    if (!token || !task) return;

    try {
      await adminCommands.toggleScheduledTask(task.id, !task.enabled, fetch, token);
      toastStore.push({ variant: "success", message: task.enabled ? "Task disabled" : "Task enabled" });
      reloadRevision += 1;
    } catch {
      toastStore.push({ variant: "error", message: "Failed to toggle task" });
    }
  }

  async function handleTrigger() {
    const token = auth.getToken();
    if (!token || !task) return;

    try {
      const result = await adminCommands.triggerScheduledTask(task.id, fetch, token);
      toastStore.push({ variant: "success", message: "Job created" });
      goto(`/system/jobs/${result.jobId}`);
    } catch {
      toastStore.push({ variant: "error", message: "Failed to trigger task" });
    }
  }

  function handleRefresh() {
    reloadRevision += 1;
    if (activeTab === "job-runs") {
      void jobsData.refetch();
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
      case "succeeded":
        return "success";
      case "failed":
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

  const jobColumns: TableColumn[] = [
    { id: "status", label: "Status", width: "minmax(100px, 1fr)" },
    { id: "attempts", label: "Attempts", width: "minmax(80px, 1fr)", align: "center" },
    { id: "createdAt", label: "Created", width: "minmax(120px, 1fr)" },
    { id: "finishedAt", label: "Finished", width: "minmax(120px, 1fr)" }
  ];

  const headerMeta = $derived.by(() => [
    { label: "ID", value: idSnippet as never },
    { label: "", value: enabledSnippet as never, separator: false }
  ]);

  const detailTabs = $derived.by(() => [
    { id: "details", label: "Details", content: detailsTabSnippet as never },
    { id: "job-runs", label: "Job Runs", content: jobRunsTabSnippet as never }
  ]);
</script>

<EntityDetailPage
  title={task ? formatTaskName(task.name) : "Scheduled Task"}
  section="Scheduled Task"
  backHref="/system/scheduled-tasks"
  backLabel="Back to tasks"
  dataLoader={taskLoader}
  reloadKey={reloadRevision}
  meta={headerMeta as never}
  headerActions={headerActionsSnippet as never}
  tabs={detailTabs as never}
  tabsVariant="card"
  tabsSize="sm"
  keepMountedTabs
  onTabChange={(tabId) => {
    activeTab = tabId;
  }}
/>

{#snippet idSnippet()}
  {#if task}
    <PoodleCode inline inlineVariant="plain" typography="inline" source={task.id} showCopyButton />
  {/if}
{/snippet}

{#snippet enabledSnippet()}
  {#if task}
    <PoodlePill tone={task.enabled ? "success" : "neutral"} appearance="badge" size="sm" typography="inherit">
      {task.enabled ? "Enabled" : "Disabled"}
    </PoodlePill>
  {/if}
{/snippet}

{#snippet headerActionsSnippet()}
  <PoodleMenu items={menuItems} ariaLabel="Task actions" placement="bottom-end" on:action={(event) => handleMenuAction(event.detail.value)}>
    <PoodleIconButton slot="trigger" icon="ellipsis" ariaLabel="Task actions" />
  </PoodleMenu>
{/snippet}

{#snippet detailsTabSnippet()}
  {#if task}
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
              <dd>{formatDisplayDateTime(task.lastScheduledAt) || "Never"}</dd>
              <dt>Last Completed</dt>
              <dd>{formatDisplayDateTime(task.lastCompletedAt) || "Never"}</dd>
              <dt>Created</dt>
              <dd>{formatDisplayDateTime(task.createdAt) || "Never"}</dd>
              <dt>Updated</dt>
              <dd>{formatDisplayDateTime(task.updatedAt) || "Never"}</dd>
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
  {/if}
{/snippet}

{#snippet jobRunsTabSnippet()}
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
{/snippet}

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
