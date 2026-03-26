<script lang="ts">
  import { PageHeader as PoodlePageHeader } from "@poodle/svelte-composites";
  import { Callout as PoodleCallout } from "@poodle/svelte-primitives";
  import { adminCommands, type ErrorLogSummary, type ErrorLogStats, type ErrorLogDetail } from "acme-client";
  import { auth } from "$lib/stores/auth";
  import { useAuthenticatedData, useToasts } from "@decodelabs/underlay/patterns";
  import { PageLoading, DataTable, TimeAgo, type DataTableColumn } from "@decodelabs/underlay/components";
  import {
    Button as PoodleButton,
    Card as PoodleCard,
    Field as PoodleField,
    Pill as PoodlePill,
    Select as PoodleSelect
  } from "@poodle/svelte-primitives";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import AlertTriangle from "lucide-svelte/icons/alert-triangle";
  import AlertCircle from "lucide-svelte/icons/alert-circle";
  import XCircle from "lucide-svelte/icons/x-circle";
  import ChevronDown from "lucide-svelte/icons/chevron-down";
  import ChevronUp from "lucide-svelte/icons/chevron-up";

  const toastStore = useToasts();

  // Filter state
  let statusCodeFilter = $state<string>("");

  // Expanded row state
  let expandedLogId = $state<string | null>(null);
  let expandedLogDetail = $state<ErrorLogDetail | null>(null);
  let loadingDetail = $state(false);

  // Fetch error logs and stats
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const [logsResponse, stats] = await Promise.all([
        adminCommands.listErrorLogs(fetch, token, {
          status_code: statusCodeFilter ? parseInt(statusCodeFilter) : undefined,
          limit: 50
        }),
        adminCommands.getErrorLogStats(fetch, token)
      ]);
      return { logs: logsResponse.data, total: logsResponse.total, stats };
    },
    {
      defaultValue: { logs: [] as ErrorLogSummary[], total: 0, stats: null as ErrorLogStats | null }
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
    void statusCodeFilter;
    if (hasFetched) {
      pageData.refetch();
    }
  });

  const logs = $derived(pageData.data?.logs ?? []);
  const stats = $derived(pageData.data?.stats);

  async function toggleDetail(logId: string) {
    if (expandedLogId === logId) {
      expandedLogId = null;
      expandedLogDetail = null;
      return;
    }

    const token = auth.getToken();
    if (!token) return;

    expandedLogId = logId;
    loadingDetail = true;
    expandedLogDetail = null;

    try {
      expandedLogDetail = await adminCommands.getErrorLog(logId, fetch, token);
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to load error details";
      toastStore.push({ variant: "error", message });
      expandedLogId = null;
    } finally {
      loadingDetail = false;
    }
  }

  function getStatusTone(statusCode: number): "neutral" | "danger" {
    if (statusCode >= 500) return "danger";
    return "neutral";
  }

  function formatDateTime(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleString();
  }

  const statusOptions = [
    { value: "", label: "All status codes" },
    { value: "400", label: "400 Bad Request" },
    { value: "401", label: "401 Unauthorized" },
    { value: "403", label: "403 Forbidden" },
    { value: "404", label: "404 Not Found" },
    { value: "500", label: "500 Server Error" },
    { value: "502", label: "502 Bad Gateway" },
    { value: "503", label: "503 Unavailable" }
  ];

  const columns: DataTableColumn<ErrorLogSummary>[] = [
    { key: "expand", label: "", width: "64px", align: "center", hideable: false },
    { key: "occurredAt", label: "Time", width: "minmax(120px, 160px)" },
    { key: "statusCode", label: "Status", width: "minmax(90px, 110px)" },
    { key: "endpoint", label: "Endpoint", width: "minmax(240px, 420px)" },
    { key: "errorCode", label: "Error Code", width: "minmax(140px, 200px)" },
    { key: "message", label: "Message", width: "minmax(220px, 420px)" }
  ];
</script>

<PoodlePageHeader title="Error Log" backHref="/system" backLabel="Back to system">
  <svelte:fragment slot="actions">
    <PoodleButton type="button" variant="ghost" on:click={() => pageData.refetch()}>
      <RefreshCw size={16} />
      Refresh
    </PoodleButton>
  </svelte:fragment>
</PoodlePageHeader>

{#if pageData.loading && logs.length === 0}
  <PageLoading message="Loading error logs..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else}
  <!-- Stats cards -->
  {#if stats}
    <div class="stats-grid">
      <PoodleCard>
        <div class="stat">
          <span class="stat-icon" style="color: var(--danger, #ef4444);">
            <AlertTriangle size={24} />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.totalLast24h}</span>
            <span class="stat-label">Total (24h)</span>
          </div>
        </div>
      </PoodleCard>
      <PoodleCard>
        <div class="stat">
          <span class="stat-icon" style="color: var(--danger, #ef4444);">
            <XCircle size={24} />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.serverErrorsLast24h}</span>
            <span class="stat-label">5xx Errors</span>
          </div>
        </div>
      </PoodleCard>
      <PoodleCard>
        <div class="stat">
          <span class="stat-icon" style="color: var(--warning, #f59e0b);">
            <AlertCircle size={24} />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.clientErrorsLast24h}</span>
            <span class="stat-label">4xx Errors</span>
          </div>
        </div>
      </PoodleCard>
    </div>
  {/if}

  <!-- Filter -->
  <div class="filter-bar">
    <PoodleField id="system-errors-status-filter" label="Status code" let:describedBy>
      <PoodleSelect
        id="system-errors-status-filter"
        value={statusCodeFilter}
        describedBy={describedBy}
        options={statusOptions}
        on:valueChange={(event) => {
          statusCodeFilter = event.detail.value;
        }}
      />
    </PoodleField>
  </div>

  <!-- Error logs list -->
  <div class="logs-list">
    <DataTable
      data={logs}
      {columns}
      loading={pageData.loading}
      emptyMessage="No error logs found"
      showLimitSelector={false}
      extendedRowWhen={(row) => expandedLogId === row.id}
    >
      {#snippet cell({ column, row })}
        {#if column.key === "expand"}
          <button
            type="button"
            class="expand-btn"
            onclick={() => toggleDetail(row.id)}
            aria-expanded={expandedLogId === row.id}
          >
            {#if expandedLogId === row.id}
              <ChevronUp size={16} />
            {:else}
              <ChevronDown size={16} />
            {/if}
          </button>
        {:else if column.key === "occurredAt"}
          <span class="time">
            <TimeAgo date={row.occurredAt} tooltipFormat="datetime" short />
          </span>
        {:else if column.key === "statusCode"}
          <PoodlePill tone={getStatusTone(row.statusCode)} appearance="badge" size="lg">
            {row.statusCode}
          </PoodlePill>
        {:else if column.key === "endpoint"}
          <div class="endpoint">
            <code class="method">{row.method}</code>
            <code class="path">{row.endpoint}</code>
          </div>
        {:else if column.key === "errorCode"}
          <code class="error-code">{row.errorCode}</code>
        {:else if column.key === "message"}
          <span class="message">{row.message || "—"}</span>
        {:else}
          —
        {/if}
      {/snippet}
      {#snippet extendedRow({ row })}
        {#if expandedLogId === row.id}
          {#if loadingDetail}
            <div class="detail-loading">Loading details...</div>
          {:else if expandedLogDetail}
            <div class="detail-content">
              <div class="detail-grid">
                <div class="detail-item">
                  <span class="detail-label">Full Timestamp</span>
                  <span class="detail-value">{formatDateTime(expandedLogDetail.occurredAt)}</span>
                </div>
                <div class="detail-item">
                  <span class="detail-label">Correlation ID</span>
                  <code class="detail-value correlation-id">{expandedLogDetail.correlationId}</code>
                </div>
                <div class="detail-item">
                  <span class="detail-label">Full Endpoint</span>
                  <code class="detail-value">{expandedLogDetail.method} {expandedLogDetail.endpoint}</code>
                </div>
                <div class="detail-item">
                  <span class="detail-label">Error Code</span>
                  <code class="detail-value">{expandedLogDetail.errorCode}</code>
                </div>
              </div>
              {#if expandedLogDetail.message}
                <div class="detail-item detail-item--full">
                  <span class="detail-label">Message</span>
                  <span class="detail-value">{expandedLogDetail.message}</span>
                </div>
              {/if}
              {#if expandedLogDetail.context && Object.keys(expandedLogDetail.context).length > 0}
                <div class="detail-item detail-item--full">
                  <span class="detail-label">Context</span>
                  <pre class="detail-context">{JSON.stringify(expandedLogDetail.context, null, 2)}</pre>
                </div>
              {/if}
            </div>
          {/if}
        {/if}
      {/snippet}
      {#snippet empty()}
        <div class="empty-state">
          <AlertCircle size={32} />
          <p>No error logs found</p>
        </div>
      {/snippet}
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

  .logs-list {
    background: var(--admin-color-surface-card);
    border-radius: 0.5rem;
  }

  .logs-list :global(.underlay-data-table) {
    --underlay-table-border: 1px solid var(--admin-color-border-subtle);
    --underlay-table-header-bg: var(--admin-color-surface-subtle);
    --underlay-table-row-hover: var(--admin-color-surface-subtle);
    --underlay-table-row-selected: var(--admin-color-surface-subtle);
    color: var(--admin-color-text);
  }

  .logs-list :global(.underlay-data-table-wrapper) {
    border-radius: 0.5rem;
  }

  .logs-list :global(.underlay-table-cell) {
    padding: 0.75rem 1rem;
  }

  .logs-list :global(.underlay-table-body > .underlay-table-row.underlay-has-extended > .underlay-table-cell) {
    background: var(--admin-color-surface-subtle);
  }

  .logs-list :global(.underlay-table-row--extended > .underlay-table-cell) {
    padding: 0;
    background: var(--admin-color-surface-subtle);
  }

  .logs-list :global(.underlay-empty-state) {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 3rem;
    color: var(--admin-color-text-muted);
  }

  .expand-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 28px;
    padding: 0;
    background: transparent;
    border: 1px solid var(--admin-color-border-subtle);
    border-radius: 4px;
    color: var(--admin-color-text-muted);
    cursor: pointer;
    margin: 0 auto;
  }

  .expand-btn:hover {
    background: var(--admin-color-surface-subtle);
    color: var(--admin-color-text);
  }

  .time {
    font-size: 0.875rem;
    color: var(--admin-color-text-muted);
    white-space: nowrap;
  }

  .endpoint {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .method {
    font-family: monospace;
    font-size: 0.7rem;
    background: var(--admin-color-surface-subtle);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    font-weight: 600;
    color: var(--admin-color-text);
  }

  .path {
    font-family: monospace;
    font-size: 0.8rem;
    color: var(--admin-color-text-muted);
  }

  .error-code {
    font-family: monospace;
    font-size: 0.8rem;
    background: var(--admin-color-surface-subtle);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    color: var(--admin-color-text);
  }

  .message {
    font-size: 0.875rem;
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .detail-loading {
    padding: 1.5rem;
    text-align: center;
    color: var(--admin-color-text-muted);
  }

  .detail-content {
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .detail-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
  }

  .detail-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .detail-item--full {
    grid-column: 1 / -1;
  }

  .detail-label {
    font-size: 0.7rem;
    font-weight: 600;
    color: var(--admin-color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .detail-value {
    font-size: 0.85rem;
    color: var(--admin-color-text);
  }

  code.detail-value {
    font-family: monospace;
    font-size: 0.8rem;
    background: var(--admin-color-surface-card);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    display: inline-block;
    color: var(--admin-color-text);
  }

  .correlation-id {
    font-size: 0.75rem;
    word-break: break-all;
  }

  .detail-context {
    font-family: monospace;
    font-size: 0.8rem;
    background: var(--admin-color-surface-subtle, #1e293b);
    color: var(--admin-color-text, #e2e8f0);
    padding: 1rem;
    border-radius: 0.5rem;
    overflow-x: auto;
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
