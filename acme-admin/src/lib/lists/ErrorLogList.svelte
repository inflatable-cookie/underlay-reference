<script lang="ts">
  import type { QueryParams } from "@decodelabs/underlay/client/query";
  import { useAuthenticatedData } from "@decodelabs/underlay/runtime/auth";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";
  import { EntityListPage, toPagedListResult } from "@decodelabs/underlay/templates";
  import {
    Card as PoodleCard,
    Icon,
    Pill as PoodlePill,
    TimeAgo,
    formatDisplayDateTime,
    type TableColumn,
    type TableRow
  } from "@poodle/svelte";
  import {
    adminCommands,
    type ErrorLogDetail,
    type ErrorLogStats,
    type ErrorLogSummary
  } from "@api-client";
  import { auth } from "$lib/stores/auth";

  interface Props {
    title?: string;
    backHref?: string;
    backLabel?: string;
    query: QueryParams;
    onQueryChange: (query: QueryParams) => void;
  }

  let {
    title = "Error Log",
    backHref = "/system",
    backLabel = "Back to system",
    query,
    onQueryChange
  }: Props = $props();

  const toastStore = useToasts();

  let expandedLogId = $state<string | null>(null);
  let expandedLogDetail = $state<ErrorLogDetail | null>(null);
  let loadingDetail = $state(false);

  const statsData = useAuthenticatedData(
    async (fetch, token) => {
      if (!token) throw new Error("Not authenticated");
      return await adminCommands.getErrorLogStats(fetch, token);
    },
    {
      defaultValue: null as ErrorLogStats | null
    }
  );

  const stats = $derived(statsData.data);

  const columns: TableColumn[] = [
    { id: "expand", label: "", width: "3.5rem", align: "center", hideable: false, isRowHeader: false },
    { id: "occurredAt", label: "Time", width: "9rem" },
    { id: "statusCode", label: "Status", width: "7rem" },
    { id: "endpoint", label: "Endpoint", width: "minmax(15rem, 24rem)" },
    { id: "errorCode", label: "Error Code", width: "minmax(9rem, 12rem)" },
    { id: "message", label: "Message", width: "minmax(14rem, 26rem)" }
  ];

  const filters = [
    {
      id: "statusCode",
      type: "select" as const,
      label: "Status",
      options: [
        { value: "All", label: "All statuses" },
        { value: "400", label: "400 Bad Request" },
        { value: "401", label: "401 Unauthorized" },
        { value: "403", label: "403 Forbidden" },
        { value: "404", label: "404 Not Found" },
        { value: "500", label: "500 Server Error" },
        { value: "502", label: "502 Bad Gateway" },
        { value: "503", label: "503 Unavailable" }
      ]
    }
  ];

  function getStatusCodeFilter(nextQuery: QueryParams): number | undefined {
    const filter = nextQuery.filters?.find((entry) => entry.field === "statusCode");
    if (!filter || filter.value === "" || filter.value === "All") {
      return undefined;
    }

    const parsed = Number(filter.value);
    return Number.isFinite(parsed) ? parsed : undefined;
  }

  async function dataLoader(fetch: typeof window.fetch, token: string | null, nextQuery: QueryParams) {
    if (!token) throw new Error("Not authenticated");

    const limit = nextQuery.limit ?? 30;
    const pageNumber = Math.max(1, nextQuery.page ?? 1);
    const offset = (pageNumber - 1) * limit;

    const response = await adminCommands.listErrorLogs(fetch, token, {
      status_code: getStatusCodeFilter(nextQuery),
      limit,
      offset
    });
    return toPagedListResult(response);
  }

  function getStatusTone(statusCode: number): "neutral" | "danger" {
    return statusCode >= 500 ? "danger" : "neutral";
  }

  function getRowLog(row: TableRow<ErrorLogSummary>): ErrorLogSummary | null {
    return row.data ?? null;
  }

  async function toggleDetail(logId: string) {
    if (expandedLogId === logId) {
      expandedLogId = null;
      expandedLogDetail = null;
      return;
    }

    const token = auth.getToken();
    if (!token) return;

    expandedLogId = logId;
    expandedLogDetail = null;
    loadingDetail = true;

    try {
      expandedLogDetail = await adminCommands.getErrorLog(logId, fetch, token);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to load error details";
      toastStore.push({ variant: "error", message });
      expandedLogId = null;
    } finally {
      loadingDetail = false;
    }
  }
</script>

{#snippet beforeList()}
  {#if stats}
    <div class="stats-grid">
      <PoodleCard>
        <div class="stat">
          <span class="stat-icon stat-icon--danger">
            <Icon icon="triangle-alert" size="lg" />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.totalLast24h}</span>
            <span class="stat-label">Total Errors</span>
          </div>
        </div>
      </PoodleCard>
      <PoodleCard>
        <div class="stat">
          <span class="stat-icon stat-icon--danger">
            <Icon icon="circle-x" size="lg" />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.serverErrorsLast24h}</span>
            <span class="stat-label">5xx Errors</span>
          </div>
        </div>
      </PoodleCard>
      <PoodleCard>
        <div class="stat">
          <span class="stat-icon stat-icon--warning">
            <Icon icon="alert-circle" size="lg" />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.clientErrorsLast24h}</span>
            <span class="stat-label">4xx Errors</span>
          </div>
        </div>
      </PoodleCard>
    </div>
  {/if}
{/snippet}

{#snippet renderCell(column: TableColumn, row: TableRow<ErrorLogSummary>, value: string | number | null)}
  {@const log = getRowLog(row)}
  {#if !log}
    —
  {:else if column.id === "expand"}
    <button
      type="button"
      class="expand-btn"
      onclick={() => toggleDetail(row.id)}
      aria-label={expandedLogId === row.id ? "Collapse error details" : "Expand error details"}
      aria-expanded={expandedLogId === row.id}
    >
      <Icon icon={expandedLogId === row.id ? "chevron-up" : "chevron-down"} size="sm" />
    </button>
  {:else if column.id === "occurredAt"}
    <span class="time">
      <TimeAgo datetime={log.occurredAt} tooltipFormat="datetime" short />
    </span>
  {:else if column.id === "statusCode"}
    <PoodlePill tone={getStatusTone(log.statusCode)} appearance="badge" size="sm">
      {log.statusCode}
    </PoodlePill>
  {:else if column.id === "endpoint"}
    <div class="endpoint">
      <code class="method">{log.method}</code>
      <code class="path">{log.endpoint}</code>
    </div>
  {:else if column.id === "errorCode"}
    <code class="error-code">{log.errorCode}</code>
  {:else if column.id === "message"}
    <span class="message">{log.message || "—"}</span>
  {:else}
    {value ?? "—"}
  {/if}
{/snippet}

{#snippet renderExpandedRow(row: TableRow<ErrorLogSummary>)}
  {@const log = getRowLog(row)}
  {#if log && expandedLogId === row.id}
    {#if loadingDetail}
      <div class="detail-loading">Loading details...</div>
    {:else if expandedLogDetail}
      <div class="detail-content">
        <div class="detail-grid">
          <div class="detail-item">
            <span class="detail-label">Full Timestamp</span>
            <span class="detail-value">{formatDisplayDateTime(expandedLogDetail.occurredAt)}</span>
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

<EntityListPage
  {title}
  {backHref}
  {backLabel}
  {dataLoader}
  presentation="table"
  {columns}
  {filters}
  {query}
  {onQueryChange}
  beforeList={beforeList}
  renderCell={renderCell as never}
  renderExpandedRow={renderExpandedRow as never}
/>

<style>
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr));
    gap: 1rem;
  }

  .stat {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem;
  }

  .stat-icon {
    display: flex;
    flex-shrink: 0;
  }

  .stat-icon--warning {
    color: var(--admin-color-warning, #f59e0b);
  }

  .stat-icon--danger {
    color: var(--admin-color-danger, #ef4444);
  }

  .stat-content {
    display: flex;
    flex-direction: column;
  }

  .stat-value {
    color: var(--poodle-color-text);
    font-size: 1.5rem;
    font-weight: 600;
  }

  .stat-label {
    color: var(--poodle-color-text-muted);
    font-size: 0.875rem;
  }

  .expand-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: 0;
    background: transparent;
    cursor: pointer;
    color: inherit;
  }

  .endpoint {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .method,
  .path,
  .error-code,
  .correlation-id,
  .detail-context {
    font-family: var(--poodle-font-family-mono, monospace);
  }

  .message {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .detail-loading {
    padding: 1rem;
    color: var(--poodle-color-text-muted);
  }

  .detail-content {
    padding: 1rem;
    background: var(--poodle-color-surface-subtle);
    border-radius: 0.75rem;
  }

  .detail-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
    gap: 1rem;
  }

  .detail-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .detail-item--full {
    margin-top: 1rem;
  }

  .detail-label {
    font-size: 0.875rem;
    color: var(--poodle-color-text-muted);
  }

  .detail-context {
    margin: 0;
    white-space: pre-wrap;
  }
</style>
