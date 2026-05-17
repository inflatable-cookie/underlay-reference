<script lang="ts">
  import { auth } from "$lib/stores/auth";
  import { EntityDetailPage } from "@decodelabs/underlay/templates";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";
  import {
    Button as PoodleButton,
    Card as PoodleCard,
    Code,
    DetailItem as PoodleDetailItem,
    DetailSection as PoodleDetailSection,
    IconButton as PoodleIconButton,
    Pill as PoodlePill,
    formatDisplayDateTime
  } from "@poodle/svelte";
  import RotateCcw from "lucide-svelte/icons/rotate-ccw";
  import Ban from "lucide-svelte/icons/ban";
  import { adminCommands, type JobDetail } from "@api-client";
  import { getJobStatusTone } from "$lib/utils/accents";
  import { refreshCwIcon } from "$lib/ui/poodle-icon-nodes";

  interface Props {
    data: { id: string };
  }

  let { data }: Props = $props();

  const toastStore = useToasts();
  let job = $state<JobDetail | null>(null);
  let reloadRevision = $state(0);

  async function jobLoader(fetch: typeof window.fetch, token: string | null) {
    if (!token) throw new Error("Not authenticated");
    const result = await adminCommands.getJob(data.id, fetch, token);
    job = result;
    return result;
  }

  function formatJobType(jobType: string): string {
    return jobType
      .replace(/_/g, " ")
      .replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function getStatusLabel(status: string): string {
    return status.charAt(0).toUpperCase() + status.slice(1);
  }

  async function handleCancel() {
    const token = auth.getToken();
    if (!token || !job) return;

    try {
      await adminCommands.cancelJob(job.id, fetch, token);
      toastStore.push({ variant: "success", message: "Job cancelled" });
      reloadRevision += 1;
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to cancel job";
      toastStore.push({ variant: "error", message });
    }
  }

  async function handleRetry() {
    const token = auth.getToken();
    if (!token || !job) return;

    try {
      await adminCommands.retryJob(job.id, fetch, token);
      toastStore.push({ variant: "success", message: "Job queued for retry" });
      reloadRevision += 1;
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to retry job";
      toastStore.push({ variant: "error", message });
    }
  }

  function handleRefresh() {
    reloadRevision += 1;
  }

  const headerMeta = $derived.by(() => [
    { label: "ID", value: idSnippet as never },
    { label: "", value: statusSnippet as never, separator: false }
  ]);
</script>

<EntityDetailPage
  title={job ? formatJobType(job.jobType) : "Job"}
  section="Job"
  backHref="/system/jobs"
  backLabel="Back to jobs"
  dataLoader={jobLoader}
  reloadKey={reloadRevision}
  meta={headerMeta as never}
  headerActions={headerActionsSnippet as never}
  content={detailsContentSnippet as never}
/>

{#snippet idSnippet()}
  {#if job}
    <Code inline inlineVariant="plain" typography="inline" source={job.id} showCopyButton />
  {/if}
{/snippet}

{#snippet statusSnippet()}
  {#if job}
    <PoodlePill tone={getJobStatusTone(job.status)} appearance="badge" size="sm" typography="inherit">
      {getStatusLabel(job.status)}
    </PoodlePill>
  {/if}
{/snippet}

{#snippet headerActionsSnippet()}
  {#if job}
    {#if job.status === "pending" || job.status === "running"}
      <PoodleButton variant="secondary" onClick={handleCancel}>
        <Ban size={16} />
        Cancel
      </PoodleButton>
    {/if}
    {#if job.status === "failed" || job.status === "cancelled"}
      <PoodleButton variant="primary" onClick={handleRetry}>
        <RotateCcw size={16} />
        Retry
      </PoodleButton>
    {/if}
    <PoodleIconButton
      variant="secondary"
      icon={refreshCwIcon}
      ariaLabel="Refresh job"
      tooltip="Refresh"
      onClick={handleRefresh}
    />
  {/if}
{/snippet}

{#snippet detailsContentSnippet()}
  {#if job}
    <div class="job-detail">
      <PoodleCard>
        <div class="detail-card-grid">
          <PoodleDetailSection title="Details" columns={2} separated={false}>
            <PoodleDetailItem presentation="surface" label="Type">
              {#snippet valueContent()}<Code inline source={job.jobType} />{/snippet}
            </PoodleDetailItem>
            <PoodleDetailItem presentation="surface" label="Attempts" value={`${job.attempts} / ${job.maxAttempts}`} />
          </PoodleDetailSection>
          <PoodleDetailSection title="Timestamps" columns={2} separated={false}>
            <PoodleDetailItem presentation="surface" label="Created" value={formatDisplayDateTime(job.createdAt) || "-"} />
            {#if job.scheduledFor}
              <PoodleDetailItem presentation="surface" label="Scheduled for" value={formatDisplayDateTime(job.scheduledFor) || "-"} />
            {/if}
            <PoodleDetailItem presentation="surface" label="Started at" value={formatDisplayDateTime(job.startedAt) || "-"} />
            <PoodleDetailItem presentation="surface" label="Finished at" value={formatDisplayDateTime(job.finishedAt) || "-"} />
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
{/snippet}

<style>
  .job-detail {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
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
