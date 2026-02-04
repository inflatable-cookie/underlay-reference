<script lang="ts">
  import { adminCommands, type ErrorLogSummary, type ErrorLogStats, type ErrorLogDetail } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import { useAuthenticatedData, PageHeader, useToasts } from "@decodelabs/underlay/patterns";
  import { Button, PageLoading, FormError, Badge, Select, Card } from "@decodelabs/underlay/components";
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

  function formatRelativeTime(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d ago`;
    if (hours > 0) return `${hours}h ago`;
    if (minutes > 0) return `${minutes}m ago`;
    return "just now";
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
</script>

<PageHeader title="Error log" backHref="/system" backLabel="Back to system">
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
    {#if logs.length === 0}
      <div class="empty-state">
        <AlertCircle size={32} />
        <p>No error logs found</p>
      </div>
    {:else}
      <table class="logs-table">
        <thead>
          <tr>
            <th></th>
            <th>Time</th>
            <th>Status</th>
            <th>Endpoint</th>
            <th>Error Code</th>
            <th>Message</th>
          </tr>
        </thead>
        <tbody>
          {#each logs as log (log.id)}
            <tr class="log-row" class:log-row--expanded={expandedLogId === log.id}>
              <td class="expand-cell">
                <button
                  type="button"
                  class="expand-btn"
                  onclick={() => toggleDetail(log.id)}
                  aria-expanded={expandedLogId === log.id}
                >
                  {#if expandedLogId === log.id}
                    <ChevronUp size={16} />
                  {:else}
                    <ChevronDown size={16} />
                  {/if}
                </button>
              </td>
              <td class="time">
                <span title={formatDateTime(log.occurredAt)}>
                  {formatRelativeTime(log.occurredAt)}
                </span>
              </td>
              <td>
                <Badge variant={getStatusVariant(log.statusCode)} size="sm">
                  {log.statusCode}
                </Badge>
              </td>
              <td>
                <div class="endpoint">
                  <code class="method">{log.method}</code>
                  <code class="path">{log.endpoint}</code>
                </div>
              </td>
              <td>
                <code class="error-code">{log.errorCode}</code>
              </td>
              <td class="message">
                {log.message || "—"}
              </td>
            </tr>
            {#if expandedLogId === log.id}
              <tr class="detail-row">
                <td colspan="6">
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
                </td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
    {/if}
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
    border: 1px solid var(--admin-color-border-subtle);
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 3rem;
    color: var(--admin-color-text-muted);
  }

  .logs-table {
    width: 100%;
    border-collapse: collapse;
  }

  .logs-table th,
  .logs-table td {
    padding: 0.75rem 1rem;
    text-align: left;
    border-bottom: 1px solid var(--admin-color-border-subtle);
  }

  .logs-table th {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--admin-color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    background: var(--admin-color-surface-subtle);
  }

  .log-row {
    cursor: pointer;
  }

  .log-row:hover {
    background: var(--admin-color-surface-subtle);
  }

  .log-row--expanded {
    background: var(--admin-color-surface-subtle);
  }

  .expand-cell {
    width: 40px;
    padding: 0.5rem !important;
  }

  .expand-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    padding: 0;
    background: transparent;
    border: 1px solid var(--admin-color-border-subtle);
    border-radius: 4px;
    color: var(--admin-color-text-muted);
    cursor: pointer;
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

  /* Detail row styles */
  .detail-row td {
    padding: 0 !important;
    background: var(--admin-color-surface-subtle);
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

  .detail-value code,
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
