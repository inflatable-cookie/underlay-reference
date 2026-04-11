<script lang="ts">
import {
  createClientPagination
} from "@decodelabs/underlay/runtime/data";
    import {
    Button as PoodleButton,
    Card as PoodleCard,
    Field as PoodleField,
    Pill as PoodlePill,
    TextInput as PoodleSearchField,
    Select as PoodleSelect
  } from "@poodle/svelte";
  import { ListContainer as PoodleListContainer, PageHeader as PoodlePageHeader } from "@poodle/svelte";
  import Activity from "lucide-svelte/icons/activity";
  import Layers from "lucide-svelte/icons/layers";
  import RefreshCw from "lucide-svelte/icons/refresh-cw";
  import ShieldAlert from "lucide-svelte/icons/shield-alert";

  type ReviewItem = {
    id: string;
    title: string;
    status: "healthy" | "warning" | "error";
    summary: string;
  };

  type QueueItem = {
    id: string;
    name: string;
    status: "idle" | "running" | "degraded";
    owner: string;
  };

  const reviewItems: ReviewItem[] = [
    {
      id: "wf-001",
      title: "Auth recovery emails",
      status: "healthy",
      summary: "Templates rendered and delivered inside expected latency."
    },
    {
      id: "wf-002",
      title: "Background media ingest",
      status: "warning",
      summary: "Two jobs retried after temporary storage timeouts."
    },
    {
      id: "wf-003",
      title: "Audit export archive",
      status: "error",
      summary: "Archive generation failed after the second chunk."
    },
    {
      id: "wf-004",
      title: "User invite flow",
      status: "healthy",
      summary: "Invites completed successfully across all environments."
    },
    {
      id: "wf-005",
      title: "Content restore planner",
      status: "warning",
      summary: "Planner completed, but three relations need manual review."
    },
    {
      id: "wf-006",
      title: "Invoice reminder dispatch",
      status: "healthy",
      summary: "Queue processed cleanly with no retries."
    }
  ];

  const queueItems: QueueItem[] = [
    { id: "ops-001", name: "Email capture relay", status: "running", owner: "Messaging" },
    { id: "ops-002", name: "Search indexing", status: "degraded", owner: "Platform" },
    { id: "ops-003", name: "Blob cleanup", status: "idle", owner: "Storage" },
    { id: "ops-004", name: "Nightly digest", status: "running", owner: "Product" },
    { id: "ops-005", name: "Job reaper", status: "idle", owner: "Platform" },
    { id: "ops-006", name: "Slack notifier", status: "degraded", owner: "Messaging" },
    { id: "ops-007", name: "Preview snapshotter", status: "running", owner: "Media" }
  ];

  let listQuery = $state("");
  let listStatus = $state("");
  let pageRefreshCount = $state(0);

  let gridQuery = $state("");

  const listStatusOptions = [
    { value: "", label: "All statuses" },
    { value: "healthy", label: "Healthy" },
    { value: "warning", label: "Warning" },
    { value: "error", label: "Error" }
  ];

  const filteredListItems = $derived(
    reviewItems.filter((item) => {
      const matchesQuery =
        listQuery.trim().length === 0 ||
        item.title.toLowerCase().includes(listQuery.trim().toLowerCase()) ||
        item.summary.toLowerCase().includes(listQuery.trim().toLowerCase());
      const matchesStatus = listStatus.length === 0 || item.status === listStatus;
      return matchesQuery && matchesStatus;
    })
  );

  const filteredQueueItems = $derived(
    queueItems.filter((item) => {
      return (
        gridQuery.trim().length === 0 ||
        item.name.toLowerCase().includes(gridQuery.trim().toLowerCase()) ||
        item.owner.toLowerCase().includes(gridQuery.trim().toLowerCase())
      );
    })
  );

  const listPagination = createClientPagination(() => filteredListItems, {
    pageSize: 3,
    persistKey: "acme-admin-poodle-gap-review-list"
  });

  const gridPagination = createClientPagination(() => filteredQueueItems, {
    pageSize: 4,
    persistKey: "acme-admin-poodle-gap-review-grid"
  });

  function getReviewTone(status: ReviewItem["status"]): "success" | "neutral" | "danger" {
    if (status === "healthy") return "success";
    if (status === "error") return "danger";
    return "neutral";
  }

  function getQueueTone(status: QueueItem["status"]): "success" | "neutral" | "danger" {
    if (status === "running") return "success";
    if (status === "degraded") return "danger";
    return "neutral";
  }

  function titleCase(value: string): string {
    return value.charAt(0).toUpperCase() + value.slice(1);
  }
</script>

<PoodlePageHeader
  title="Poodle Gap Review"
  subtitle="Review the remaining ambiguous Underlay workflow surfaces with real examples before pushing them into Poodle contracts or composition guidance."
  backHref="/system"
  backLabel="Back to system"
/>

<div class="review-page">
  <section class="review-page__section">
    <header class="review-page__intro">
      <h2>DiagnosticsToolbar</h2>
      <p>
        A compact operational status strip with inline metadata and action affordances.
      </p>
    </header>

    <div class="review-diagnostics-toolbar">
      <div class="review-diagnostics-toolbar__content">
        <span class="review-diagnostics-toolbar__title">Queue health</span>
        <span>Build: acme-admin@preview</span>
        <span>Latency: 184ms</span>
        <span>Retries: 2</span>
        <span>Last refresh: {pageRefreshCount}</span>
      </div>
      <div class="review-diagnostics-toolbar__actions">
        <PoodleButton type="button" variant="ghost" on:click={() => pageRefreshCount += 1}>
          <RefreshCw size={14} />
          Refresh
        </PoodleButton>
        <PoodleButton type="button" variant="ghost">
          <ShieldAlert size={14} />
          Inspect warnings
        </PoodleButton>
      </div>
    </div>
  </section>

  <section class="review-page__section">
    <header class="review-page__intro">
      <h2>ListContainer</h2>
      <p>
        A page-level list wrapper that owns header actions, filter placement, empty states, and pagination slots.
      </p>
    </header>

    <PoodleListContainer
      title="Workflow incidents"
      subtitle="A Poodle-native list shell with built-in pagination and caller-owned content."
      eyebrow="Poodle composite"
      state={filteredListItems.length > 0 ? "ready" : "empty"}
      emptyTitle="No incidents match"
      emptyMessage="Try widening the filters or clearing the search query."
      currentPage={listPagination.currentPage}
      totalPages={listPagination.totalPages ?? 1}
      totalItems={filteredListItems.length}
      pageSize={listPagination.pageSize}
      on:pageChange={(event: CustomEvent<{ page: number }>) => listPagination.goToPage?.(event.detail.page)}
    >
      <svelte:fragment slot="actions">
        <PoodleButton type="button" variant="primary">Create report</PoodleButton>
      </svelte:fragment>

      <svelte:fragment slot="filters">
        <div class="review-filters">
          <PoodleField id="review-list-search" label="Search" let:describedBy>
            <PoodleSearchField type="search"
              id="review-list-search"
              value={listQuery}
              describedBy={describedBy}
              placeholder="Search title or summary"
              on:valueChange={(event) => {
                listQuery = event.detail.value;
                void listPagination.reset();
              }}
            />
          </PoodleField>

          <PoodleField id="review-list-status" label="Status" let:describedBy>
            <PoodleSelect
              id="review-list-status"
              value={listStatus}
              describedBy={describedBy}
              options={listStatusOptions}
              on:valueChange={(event) => {
                listStatus = event.detail.value;
                void listPagination.reset();
              }}
            />
          </PoodleField>
        </div>
      </svelte:fragment>

      <svelte:fragment slot="batch">
        <div class="review-batch">
          <PoodlePill tone="neutral" appearance="badge" size="lg">3 selected</PoodlePill>
          <PoodleButton type="button" variant="ghost">Archive</PoodleButton>
          <PoodleButton type="button" variant="ghost">Export</PoodleButton>
        </div>
      </svelte:fragment>

      <div class="review-list">
        {#each listPagination.items as item}
          <PoodleCard>
            <div class="review-list-card">
              <div class="review-list-card__header">
                <strong>{item.title}</strong>
                <PoodlePill tone={getReviewTone(item.status)} appearance="badge" size="lg">
                  {titleCase(item.status)}
                </PoodlePill>
              </div>
              <p>{item.summary}</p>
            </div>
          </PoodleCard>
        {/each}
      </div>
    </PoodleListContainer>
  </section>

  <section class="review-page__section">
    <header class="review-page__intro">
      <h2>ListContainer + client pagination</h2>
      <p>
        The same Poodle list shell can host client-side pagination by driving `currentPage`, `totalPages`, and `pageChange` from a local pagination controller.
      </p>
    </header>

    <div class="review-inline-filter">
      <PoodleField id="review-grid-search" label="Queue filter" let:describedBy>
        <PoodleSearchField type="search"
          id="review-grid-search"
          value={gridQuery}
          describedBy={describedBy}
          placeholder="Search queue or owner"
          on:valueChange={(event) => {
            gridQuery = event.detail.value;
            void gridPagination.reset();
          }}
        />
      </PoodleField>
    </div>

    <PoodleListContainer
      title="Queue workers"
      subtitle="Client-side pagination with caller-owned filtering and card layout."
      eyebrow="Poodle composite"
      state={gridPagination.items.length > 0 ? "ready" : "empty"}
      emptyTitle="No queue workers match"
      emptyMessage="Try widening the queue filter."
      currentPage={gridPagination.currentPage}
      totalPages={gridPagination.totalPages ?? 1}
      totalItems={filteredQueueItems.length}
      pageSize={gridPagination.pageSize}
      on:pageChange={(event: CustomEvent<{ page: number }>) => gridPagination.goToPage?.(event.detail.page)}
    >
      <div class="ops-grid">
        {#each gridPagination.items as item}
          <PoodleCard>
            <div class="queue-card">
              <div class="queue-card__header">
                <strong>{item.name}</strong>
                <PoodlePill tone={getQueueTone(item.status)} appearance="badge" size="lg">
                  {titleCase(item.status)}
                </PoodlePill>
              </div>
              <div class="queue-card__meta">Owner: {item.owner}</div>
              <div class="queue-card__meta">Queue ID: {item.id}</div>
            </div>
          </PoodleCard>
        {/each}
      </div>
    </PoodleListContainer>
  </section>

  <section class="review-page__section">
    <header class="review-page__intro">
      <h2>Operational section pattern</h2>
      <p>
        Operational pages can compose a titled section with an icon, an optional controls rail, and arbitrary content below without a dedicated shared wrapper.
      </p>
    </header>

    <section class="review-ops-section">
      <div class="review-ops-section__header">
        <h3 class="review-ops-section__title">
          <Layers size={16} aria-hidden="true" />
          Background services
        </h3>
        <div class="review-ops-section__controls">
          <PoodleButton type="button" variant="ghost">Mute alerts</PoodleButton>
          <PoodleButton type="button" variant="primary">Restart all</PoodleButton>
        </div>
      </div>
      <div class="ops-grid">
        <PoodleCard>
          <div class="ops-card">
            <Activity size={18} />
            <div>
              <strong>Scheduler</strong>
              <p>Running 18 jobs in the last hour without drift.</p>
            </div>
          </div>
        </PoodleCard>
        <PoodleCard>
          <div class="ops-card">
            <ShieldAlert size={18} />
            <div>
              <strong>Retry worker</strong>
              <p>Three items need manual acknowledgement after retry exhaustion.</p>
            </div>
          </div>
        </PoodleCard>
      </div>
    </section>
  </section>
</div>

<style>
  .review-page {
    display: flex;
    flex-direction: column;
    gap: 1.75rem;
  }

  .review-page__section {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .review-ops-section {
    margin-top: 1rem;
  }

  .review-ops-section__header {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    justify-content: space-between;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .review-ops-section__title {
    margin: 0;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 1rem;
  }

  .review-ops-section__controls {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    width: min(100%, 16rem);
  }

  .review-page__intro h2 {
    margin: 0;
    font-size: 1rem;
  }

  .review-page__intro p {
    margin: 0.25rem 0 0;
    color: var(--admin-color-text-muted);
    max-width: 75ch;
  }

  .review-diagnostics-toolbar {
    border: 1px solid var(--underlay-color-border-subtle, rgba(148, 163, 184, 0.25));
    border-radius: var(--underlay-radius-sm, 0.45rem);
    background: var(--underlay-color-surface-muted, rgba(15, 23, 42, 0.2));
    padding: 0.45rem 0.55rem;
    display: grid;
    gap: 0.4rem;
  }

  .review-diagnostics-toolbar__content {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.76rem;
    color: var(--underlay-color-text-muted, #94a3b8);
  }

  .review-diagnostics-toolbar__title {
    font-weight: 600;
    color: var(--underlay-color-text, #e2e8f0);
    margin-right: 0.15rem;
  }

  .review-diagnostics-toolbar__actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem;
  }

  .review-filters {
    display: grid;
    grid-template-columns: minmax(0, 2fr) minmax(12rem, 1fr);
    gap: 1rem;
  }

  .review-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .review-batch {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.75rem;
  }

  .review-list-card {
    display: grid;
    gap: 0.5rem;
  }

  .review-list-card__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
  }

  .review-list-card p {
    margin: 0;
    color: var(--admin-color-text-muted);
  }

  .review-inline-filter {
    max-width: 22rem;
  }

  .queue-card {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .queue-card__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
  }

  .queue-card__meta {
    color: var(--admin-color-text-muted);
    font-size: 0.875rem;
  }

  .ops-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
    gap: 0.85rem;
  }

  .ops-card {
    display: flex;
    gap: 0.75rem;
    align-items: flex-start;
  }

  .ops-card p {
    margin: 0.2rem 0 0;
    color: var(--admin-color-text-muted);
    font-size: 0.9rem;
  }

  @media (max-width: 800px) {
    .review-filters {
      grid-template-columns: 1fr;
    }
  }
</style>
