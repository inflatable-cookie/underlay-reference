<script lang="ts">
  import { page } from "$app/stores";
  import { adminCommands, type JobDetail } from "@api-client";
  import {
    SystemJobDetailPage,
    type SystemJobDetailItem
  } from "@inflatable-cookie/underlay/templates";
  import { getJobStatusTone } from "$lib/utils/accents";

  const jobId = $derived.by(() => $page.params.id ?? "");

  function normalizeJob(job: JobDetail): SystemJobDetailItem {
    return {
      id: job.id,
      jobType: job.jobType,
      status: job.status,
      attempts: job.attempts,
      maxAttempts: job.maxAttempts,
      createdAt: job.createdAt,
      scheduledFor: job.scheduledFor,
      startedAt: job.startedAt,
      finishedAt: job.finishedAt,
      errorMessage: job.errorMessage,
      payload: job.payload,
      progress: job.progress
    };
  }

  async function dataLoader(id: string, fetch: typeof globalThis.fetch, token: string) {
    return normalizeJob(await adminCommands.getJob(id, fetch, token));
  }

  async function cancelAction(job: SystemJobDetailItem, fetch: typeof globalThis.fetch, token: string) {
    await adminCommands.cancelJob(job.id, fetch, token);
  }

  async function retryAction(job: SystemJobDetailItem, fetch: typeof globalThis.fetch, token: string) {
    await adminCommands.retryJob(job.id, fetch, token);
  }
</script>

<SystemJobDetailPage
  id={jobId}
  {dataLoader}
  {cancelAction}
  {retryAction}
  statusTone={getJobStatusTone}
/>
