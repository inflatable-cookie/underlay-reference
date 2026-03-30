<script lang="ts">
import {
  useToasts,
  useAuthenticatedData
} from "@decodelabs/underlay/runtime";
import {
  Callout as PoodleCallout,
  DetailRow as PoodleDetailRow
  } from "@poodle/svelte-primitives";
  import { DetailSection as PoodleDetailSection,
  PageHeader as PoodlePageHeader,
  PageLoading } from "@poodle/svelte-composites";
  import { page } from "$app/stores";
    import {
    Button as PoodleButton,
    Card as PoodleCard,
    Code,
    IconButton as PoodleIconButton,
    MetaBar as PoodleMetaBar,
    MetaItem as PoodleMetaItem,
    Pill as PoodlePill
  } from "@poodle/svelte-primitives";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import RotateCcw from "lucide-svelte/icons/rotate-ccw";
  import Ban from "lucide-svelte/icons/ban";
  import { adminCommands, type JobDetail } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { getJobStatusTone } from "$lib/utils/accents";
  import { refreshCwIcon } from "$lib/ui/poodle-icon-nodes";

  const toastStore = useToasts();
  const jobId = $page.params.id;

  // Fetch job detail using authenticated data pattern
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      if (!jobId) throw new Error("Job ID is required");
      const job = await adminCommands.getJob(jobId, fetch, token);
      return { job };
    },
    {
      defaultValue: { job: undefined as JobDetail | undefined }
    }
  );

  const job = $derived(pageData.data?.job);

  function getStatusLabel(status: string): string {
    return status.charAt(0).toUpperCase() + status.slice(1);
  }

  function formatDate(dateStr: string | null | undefined): string {
    if (!dateStr) return "-";
    return new Date(dateStr).toLocaleString();
  }

  function formatJobType(jobType: string): string {
    return jobType
      .replace(/_/g, " ")
      .replace(/\b\w/g, (c) => c.toUpperCase());
  }

  async function handleCancel() {
    const token = auth.getToken();
    if (!token || !jobId) return;

    try {
      await adminCommands.cancelJob(jobId, fetch, token);
      toastStore.push({ variant: "success", message: "Job cancelled" });
      pageData.refetch();
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to cancel job";
      toastStore.push({ variant: "error", message });
    }
  }

  async function handleRetry() {
    const token = auth.getToken();
    if (!token || !jobId) return;

    try {
      await adminCommands.retryJob(jobId, fetch, token);
      toastStore.push({ variant: "success", message: "Job queued for retry" });
      pageData.refetch();
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to retry job";
      toastStore.push({ variant: "error", message });
    }
  }
</script>

<div class="job-detail__header">
<PoodlePageHeader
  section="Job"
  title={job ? formatJobType(job.jobType) : "Not Found"}
  backHref="/system/jobs"
  backLabel="Back to jobs"
>
  <svelte:fragment slot="actions">
    {#if job}
      {#if job.status === "pending" || job.status === "running"}
        <PoodleButton variant="secondary" on:click={handleCancel}>
          <Ban size={16} />
          Cancel
        </PoodleButton>
      {/if}
      {#if job.status === "failed" || job.status === "cancelled"}
        <PoodleButton variant="primary" on:click={handleRetry}>
          <RotateCcw size={16} />
          Retry
        </PoodleButton>
      {/if}
      <PoodleIconButton
        variant="secondary"
        icon={refreshCwIcon}
        ariaLabel="Refresh job"
        tooltip="Refresh"
        on:click={() => pageData.refetch()}
      />
    {/if}
  </svelte:fragment>
</PoodlePageHeader>
{#if job}
  <PoodleMetaBar ariaLabel="Job metadata">
    <PoodleMetaItem label="ID">
      <Code inline source={job.id} showCopyButton />
    </PoodleMetaItem>
    <PoodlePill tone={getJobStatusTone(job.status)} appearance="badge" size="lg">
      {getStatusLabel(job.status)}
    </PoodlePill>
  </PoodleMetaBar>
{/if}
</div>

{#if pageData.loading && !job}
  <PageLoading presentation="inline" message="Loading job details..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if job}
  <div class="job-detail">
  <PoodleCard>
    <div class="detail-card-grid">
      <PoodleDetailSection title="Details" columns={2} separated={false}>
        <PoodleDetailRow label="Type">
          <svelte:fragment slot="value"><Code inline source={job.jobType} /></svelte:fragment>
        </PoodleDetailRow>
        <PoodleDetailRow label="Attempts" value={`${job.attempts} / ${job.maxAttempts}`} />
      </PoodleDetailSection>
      <PoodleDetailSection title="Timestamps" columns={2} separated={false}>
        <PoodleDetailRow label="Created" value={formatDate(job.createdAt)} />
        {#if job.scheduledFor}
          <PoodleDetailRow label="Scheduled for" value={formatDate(job.scheduledFor)} />
        {/if}
        <PoodleDetailRow label="Started at" value={formatDate(job.startedAt)} />
        <PoodleDetailRow label="Finished at" value={formatDate(job.finishedAt)} />
      </PoodleDetailSection>
    </div>
  </PoodleCard>

    {#if job.errorMessage}
      <PoodleCard>
        <div class="job-detail__section job-detail__section--error">
          <h3>Error</h3>
          <p class="job-detail__error-text">{job.errorMessage}</p>
        </div>
      </PoodleCard>
    {/if}

    <PoodleCard>
      <div class="job-detail__section">
        <h3>Payload</h3>
        <pre class="job-detail__code">{JSON.stringify(job.payload, null, 2)}</pre>
      </div>
    </PoodleCard>

    {#if job.progress}
      <PoodleCard>
        <div class="job-detail__section">
          <h3>Progress</h3>
          <pre class="job-detail__code">{JSON.stringify(job.progress, null, 2)}</pre>
        </div>
      </PoodleCard>
    {/if}
  </div>
{/if}

<style>
  .job-detail {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .job-detail__header {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .detail-card-grid {
    display: grid;
    gap: 1rem;
  }

  .job-detail__section {
    padding: 0.5rem;
  }

  .job-detail__section h3 {
    margin: 0 0 0.75rem 0;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--admin-color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .job-detail__section--error {
    border-left: 3px solid var(--admin-color-danger);
    padding-left: 1rem;
  }

  .job-detail__error-text {
    margin: 0;
    color: var(--admin-color-danger);
    font-family: monospace;
    font-size: 0.875rem;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .job-detail__code {
    margin: 0;
    padding: 0.75rem;
    background: var(--admin-color-surface-subtle);
    border-radius: 0.35rem;
    font-size: 0.8rem;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
