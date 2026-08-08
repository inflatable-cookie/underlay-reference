<script lang="ts">
  import { page } from "$app/stores";
  import { adminCommands, type JobSummary, type ScheduledTaskDetail } from "@api-client";
  import {
    SystemScheduledTaskDetailPage,
    type SystemJobListItem,
    type SystemScheduledTaskDetailItem
  } from "@inflatable-cookie/underlay/templates";

  const taskId = $derived.by(() => $page.params.id ?? "");

  function normalizeTask(task: ScheduledTaskDetail): SystemScheduledTaskDetailItem {
    return {
      id: task.id,
      name: task.name,
      jobType: task.jobType,
      schedule: task.schedule,
      enabled: task.enabled,
      priority: task.priority,
      maxAttempts: task.maxAttempts,
      timeoutSeconds: task.timeoutSeconds,
      allowOverlap: task.allowOverlap,
      lastScheduledAt: task.lastScheduledAt,
      lastCompletedAt: task.lastCompletedAt,
      createdAt: task.createdAt,
      updatedAt: task.updatedAt,
      payload: task.payload
    };
  }

  function normalizeJob(job: JobSummary): SystemJobListItem {
    return {
      id: job.id,
      jobType: job.jobType,
      status: job.status,
      attempts: job.attempts,
      maxAttempts: job.maxAttempts,
      createdAt: job.createdAt,
      finishedAt: job.finishedAt,
      errorMessage: job.errorMessage
    };
  }

  async function dataLoader(id: string, fetch: typeof globalThis.fetch, token: string) {
    return normalizeTask(await adminCommands.getScheduledTask(id, fetch, token));
  }

  async function jobRunsLoader(task: SystemScheduledTaskDetailItem, fetch: typeof globalThis.fetch, token: string) {
    const jobs = await adminCommands.listJobs(fetch, token, { jobType: task.jobType, limit: 50 });
    return { data: jobs.data.map(normalizeJob), total: jobs.total, hasMore: jobs.hasMore };
  }

  async function toggleAction(task: SystemScheduledTaskDetailItem, fetch: typeof globalThis.fetch, token: string) {
    await adminCommands.toggleScheduledTask(task.id, !task.enabled, fetch, token);
  }

  async function triggerAction(task: SystemScheduledTaskDetailItem, fetch: typeof globalThis.fetch, token: string) {
    const result = await adminCommands.triggerScheduledTask(task.id, fetch, token);
    return { jobId: result.jobId };
  }
</script>

<SystemScheduledTaskDetailPage
  id={taskId}
  {dataLoader}
  {jobRunsLoader}
  {toggleAction}
  {triggerAction}
/>
