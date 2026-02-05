<script lang="ts">
  import { page } from "$app/stores";
  import {
    PageHeader,
    PageHeaderMeta,
    PageHeaderMetaRow,
    PageHeaderMetaItem,
    PageHeaderMetaSeparator,
    useToasts,
    useAuthenticatedData
  } from "@decodelabs/underlay/patterns";
  import {
    Button,
    Card,
    Code,
    FormError,
    PageLoading,
    DetailsCard,
    DetailsSection,
    DetailsItem,
    Pill,
    Tooltip
  } from "@decodelabs/underlay/components";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import RotateCcw from "lucide-svelte/icons/rotate-ccw";
  import Ban from "lucide-svelte/icons/ban";
  import { adminCommands, type JobDetail } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";

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
      getToken: () => auth.getToken(),
      defaultValue: { job: undefined as JobDetail | undefined }
    }
  );

  // Trigger fetch when auth is ready
  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  const job = $derived(pageData.data?.job);

  function getStatusLabel(status: string): string {
    return status.charAt(0).toUpperCase() + status.slice(1);
  }

  function getStatusAccent(status: string): string {
    switch (status) {
      case "succeeded": return "#10b981";
      case "failed": return "#ef4444";
      case "running": return "#3b82f6";
      case "pending": return "#f59e0b";
      case "cancelled": return "#6b7280";
      default: return "#64748b";
    }
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

<PageHeader
  title={job ? formatJobType(job.jobType) : "Job Detail"}
  backHref="/system/jobs"
  backLabel="Back to jobs"
>
  {#if job}
    <PageHeaderMeta>
      <PageHeaderMetaRow>
        <PageHeaderMetaItem label="ID">
          <Code copy>{job.id}</Code>
        </PageHeaderMetaItem>
        <PageHeaderMetaSeparator />
        <Pill accent={getStatusAccent(job.status)}>{getStatusLabel(job.status)}</Pill>
      </PageHeaderMetaRow>
    </PageHeaderMeta>
  {/if}

  {#snippet actions()}
    {#if job}
      {#if job.status === "pending" || job.status === "running"}
        <Button variant="secondary" onclick={handleCancel}>
          <Ban size={16} />
          Cancel
        </Button>
      {/if}
      {#if job.status === "failed" || job.status === "cancelled"}
        <Button variant="primary" onclick={handleRetry}>
          <RotateCcw size={16} />
          Retry
        </Button>
      {/if}
      <Tooltip content="Refresh" inline>
        {#snippet trigger()}
          <Button variant="subtle" size="icon" onclick={() => pageData.refetch()}>
            <RefreshCw size={16} />
          </Button>
        {/snippet}
      </Tooltip>
    {/if}
  {/snippet}
</PageHeader>

{#if pageData.loading && !job}
  <PageLoading message="Loading job details..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if job}
  <div class="job-detail">
  <DetailsCard>
    <DetailsSection legend="Details">
      <DetailsItem label="Type" value={job.jobType} code />
      <DetailsItem label="Attempts" value={`${job.attempts} / ${job.maxAttempts}`} />
    </DetailsSection>
    <DetailsSection legend="Timestamps">
      <DetailsItem label="Created" value={formatDate(job.createdAt)} />
      {#if job.scheduledFor}
        <DetailsItem label="Scheduled for" value={formatDate(job.scheduledFor)} />
      {/if}
      <DetailsItem label="Started at" value={formatDate(job.startedAt)} />
      <DetailsItem label="Finished at" value={formatDate(job.finishedAt)} />
    </DetailsSection>
  </DetailsCard>

    {#if job.errorMessage}
      <Card>
        <div class="job-detail__section job-detail__section--error">
          <h3>Error</h3>
          <p class="job-detail__error-text">{job.errorMessage}</p>
        </div>
      </Card>
    {/if}

    <Card>
      <div class="job-detail__section">
        <h3>Payload</h3>
        <pre class="job-detail__code">{JSON.stringify(job.payload, null, 2)}</pre>
      </div>
    </Card>

    {#if job.progress}
      <Card>
        <div class="job-detail__section">
          <h3>Progress</h3>
          <pre class="job-detail__code">{JSON.stringify(job.progress, null, 2)}</pre>
        </div>
      </Card>
    {/if}
  </div>
{/if}

<style>
  .job-detail {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
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
