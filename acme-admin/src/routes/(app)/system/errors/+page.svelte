<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { auth } from "$lib/stores/auth";
  import { EntityListPage, toPagedListResult } from "@decodelabs/underlay/templates";
  import { buildQueryString, parseQueryParams } from "@decodelabs/underlay/client/query";
  import type { QueryParams } from "@decodelabs/underlay/client/query";
  import { useAuthenticatedData } from "@decodelabs/underlay/runtime/auth";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";
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

  const toastStore = useToasts();
  const currentQuery = $derived(parseQueryParams($page.url.searchParams));

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

  function updateUrl(nextQuery: QueryParams) {
    const url = new URL($page.url);
    url.search = buildQueryString(nextQuery);
    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  function getStatusCodeFilter(query: QueryParams): number | undefined {
    const filter = query.filters?.find((entry) => entry.field === "statusCode");
    if (!filter || filter.value === "" || filter.value === "All") {
      return undefined;
    }

    const parsed = Number(filter.value);
    return Number.isFinite(parsed) ? parsed : undefined;
  }

  async function dataLoader(fetch: typeof window.fetch, token: string | null, query: QueryParams) {
    if (!token) throw new Error("Not authenticated");

    const limit = query.limit ?? 30;
    const pageNumber = Math.max(1, query.page ?? 1);
    const offset = (pageNumber - 1) * limit;

    const response = await adminCommands.listErrorLogs(fetch, token, {
      status_code: getStatusCodeFilter(query),
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
  title="Error Log"
  backHref="/system"
  backLabel="Back to system"
  {dataLoader}
  presentation="table"
  {columns}
  {filters}
  showRowActions={false}
  renderCell={renderCell as never}
  renderExpandedRow={renderExpandedRow as never}
  expandedRowIds={expandedLogId ? [expandedLogId] : []}
  query={currentQuery}
  onQueryChange={updateUrl}
  beforeList={beforeList as never}
/>

<style>
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(11rem, 1fr));
    gap: 1rem;
  }

  .stat {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem;
  }

  .stat-icon {
    display: inline-flex;
    flex-shrink: 0;
  }

  .stat-icon--danger {
    color: var(--poodle-color-status-danger);
  }

  .stat-icon--warning {
    color: var(--poodle-color-status-warning);
  }

  .stat-content {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--poodle-color-text-primary);
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--poodle-color-text-secondary);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .expand-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.75rem;
    height: 1.75rem;
    padding: 0;
    border: 0;
    border-radius: var(--poodle-radius-control);
    background: transparent;
    color: var(--poodle-color-text-secondary);
    cursor: pointer;
  }

  .expand-btn:hover,
  .expand-btn:focus-visible {
    background: color-mix(in srgb, var(--poodle-color-accent-base) 12%, transparent);
    color: var(--poodle-color-text-primary);
    outline: none;
  }

  .time {
    color: var(--poodle-color-text-secondary);
  }

  .endpoint {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .method,
  .path,
  .error-code,
  .correlation-id {
    font-family: var(--poodle-typography-code-family);
    font-size: 0.75rem;
  }

  .method {
    color: var(--poodle-color-text-secondary);
    flex-shrink: 0;
  }

  .path {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .message {
    color: var(--poodle-color-text-secondary);
  }

  .detail-loading {
    padding: 0.25rem 0;
    color: var(--poodle-color-text-secondary);
  }

  .detail-content {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .detail-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
    gap: 0.75rem 1rem;
  }

  .detail-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
  }

  .detail-item--full {
    width: 100%;
  }

  .detail-label {
    color: var(--poodle-color-text-secondary);
    font-size: 0.75rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .detail-value {
    color: var(--poodle-color-text-primary);
    min-width: 0;
  }

  .detail-context {
    margin: 0;
    padding: 0.75rem;
    border-radius: var(--poodle-radius-surface);
    background: color-mix(in srgb, var(--poodle-color-background-elevated) 94%, transparent);
    color: var(--poodle-color-text-primary);
    font-family: var(--poodle-typography-code-family);
    font-size: 0.75rem;
    line-height: 1.5;
    overflow-x: auto;
  }
</style>
