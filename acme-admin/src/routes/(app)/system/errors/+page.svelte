<script lang="ts">
  import { adminCommands, type ErrorLogSummary, type ErrorLogStats, type ErrorLogDetail } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import { useAuthenticatedData, PageHeader, useToasts } from "@decodelabs/underlay/patterns";
  import { Button, PageLoading, FormError, Badge, Select, Card, DataTable, TimeAgo, DetailsCard, DetailsSection, DetailsItem, type DataTableColumn } from "@decodelabs/underlay/components";
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
          statusCode: statusCodeFilter ? parseInt(statusCodeFilter) : undefined,
          limit: 50
        }),
        adminCommands.getErrorLogStats(fetch, token)
      ]);
      return { logs: logsResponse.data, total: logsResponse.total, stats };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { logs: [] as ErrorLogSummary[], total: 0, stats: null as ErrorLogStats | null }
    }
  );

  // Trigger fetch when auth is ready
  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  // Refetch when filter changes
  $effect(() => {
    void statusCodeFilter;
    if ($currentUser) {
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

  type BadgeVariant = "default" | "success" | "warning" | "danger" | "info" | "muted";

  function getStatusVariant(statusCode: number): BadgeVariant {
    if (statusCode >= 500) return "danger";
    if (statusCode >= 400) return "warning";
    return "default";
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

<PageHeader section="Error Log" backHref="/system" backLabel="Back to system">
  {#snippet actions()}
    <Button type="button" variant="subtle" onclick={() => pageData.refetch()}>
      <RefreshCw size={16} />
      Refresh
    </Button>
  {/snippet}
</PageHeader>

{#if pageData.loading && logs.length === 0}
  <PageLoading message="Loading error logs..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else}
  <!-- Stats cards -->
  {#if stats}
    <div class="stats-grid">
      <Card>
        <div class="stat">
          <span class="stat-icon" style="color: var(--danger, #ef4444);">
            <AlertTriangle size={24} />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.totalLast24h}</span>
            <span class="stat-label">Total (24h)</span>
          </div>
        </div>
      </Card>
      <Card>
        <div class="stat">
          <span class="stat-icon" style="color: var(--danger, #ef4444);">
            <XCircle size={24} />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.serverErrorsLast24h}</span>
            <span class="stat-label">5xx Errors</span>
          </div>
        </div>
      </Card>
      <Card>
        <div class="stat">
          <span class="stat-icon" style="color: var(--warning, #f59e0b);">
            <AlertCircle size={24} />
          </span>
          <div class="stat-content">
            <span class="stat-value">{stats.clientErrorsLast24h}</span>
            <span class="stat-label">4xx Errors</span>
          </div>
        </div>
      </Card>
    </div>
  {/if}

  <!-- Filter -->
  <div class="filter-bar">
    <Select
      bind:value={statusCodeFilter}
      items={statusOptions}
      placeholder="All status codes"
    />
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
          <Badge variant={getStatusVariant(row.statusCode)} size="sm">
            {row.statusCode}
          </Badge>
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
              <DetailsCard class="detail-card">
                <DetailsSection>
                  <DetailsItem label="Full Timestamp" value={formatDateTime(expandedLogDetail.occurredAt)} />
                  <DetailsItem label="Correlation ID" value={expandedLogDetail.correlationId} code />
                  <DetailsItem label="Full Endpoint" value={`${expandedLogDetail.method} ${expandedLogDetail.endpoint}`} code />
                  <DetailsItem label="Error Code" value={expandedLogDetail.errorCode} code />
                  {#if expandedLogDetail.message}
                    <DetailsItem label="Message" value={expandedLogDetail.message} span="full" />
                  {/if}
                  {#if expandedLogDetail.context && Object.keys(expandedLogDetail.context).length > 0}
                    <DetailsItem label="Context" span="full">
                      <pre class="detail-context">{JSON.stringify(expandedLogDetail.context, null, 2)}</pre>
                    </DetailsItem>
                  {/if}
                </DetailsSection>
              </DetailsCard>
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

  .logs-list :global(.table-cell) {
    padding: 0.75rem 1rem;
  }

  .logs-list :global(.table-body > .table-row.has-extended > .table-cell) {
    background: var(--admin-color-surface-subtle);
  }

  .logs-list :global(.table-row--extended > .table-cell) {
    padding: 0;
    background: var(--admin-color-surface-subtle);
  }

  .logs-list :global(.empty-state) {
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
    padding: 1.25rem 1.5rem;
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
