<script lang="ts">
import {
  useToasts,
  useAuthenticatedData
} from "@decodelabs/underlay/runtime";
import {
  Callout as PoodleCallout,
  Grid as PoodleGrid,
  ListCard as PoodleListCard } from "@poodle/svelte-primitives";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
    import {
    FilterToolbar,
    ListContainer
  } from "@poodle/svelte-composites";
  import {
    Button as PoodleButton,
    Field as PoodleField,
    IconButton as PoodleIconButton,
    TextInput as PoodleTextInput
  } from "@poodle/svelte-primitives";
  import Mail from "lucide-svelte/icons/mail";
  import Filter from "lucide-svelte/icons/filter";
  import { adminCommands } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import CopyActionsMenu from "$lib/components/CopyActionsMenu.svelte";
  import type { CapturedEmailSummary } from "@api-client";

  const toastStore = useToasts();

  // Track URL for refetching when filters change
  let previousUrl = $state<string | null>(null);

  // Derive filters from URL
  const filters = $derived({
    to_address: $page.url.searchParams.get("to_address") ?? "",
    from_address: $page.url.searchParams.get("from_address") ?? "",
    sinceDate: $page.url.searchParams.get("since") ?? "",
    untilDate: $page.url.searchParams.get("until") ?? ""
  });

  // Fetch captured emails using authenticated data pattern
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const toAddressParam = $page.url.searchParams.get("to_address");
      const fromAddressParam = $page.url.searchParams.get("from_address");
      const sinceDateParam = $page.url.searchParams.get("since");
      const untilDateParam = $page.url.searchParams.get("until");

      const since = sinceDateParam ? `${sinceDateParam}T00:00:00Z` : undefined;
      const until = untilDateParam ? `${untilDateParam}T23:59:59Z` : undefined;

      const entries = await adminCommands.listCapturedEmails(fetch, token, {
        to_address: toAddressParam ?? undefined,
        from_address: fromAddressParam ?? undefined,
        since,
        until
      });
      return { entries };
    },
    {
      defaultValue: { entries: [] as CapturedEmailSummary[] },
      onSuccess: () => {
        previousUrl = $page.url.search;
      }
    }
  );

  // Refetch when URL changes (for filtering)
  $effect(() => {
    const currentUrl = $page.url.search;
    if (previousUrl !== null && previousUrl !== currentUrl) {
      previousUrl = currentUrl;
      pageData.refetch();
    }
  });

  const entries = $derived(pageData.data?.entries ?? []);
  let filtersCollapsed = $state(true);

  const listState = $derived(
    pageData.loading ? "loading" : pageData.error ? "error" : entries.length === 0 ? "empty" : "ready"
  );
  const filterSummaryText = $derived(
    entries.length > 0 ? `Showing ${entries.length} captured email${entries.length === 1 ? "" : "s"}` : "No captured emails"
  );

  function openEmail(id: string) {
    void goto(`/system/emails/${encodeURIComponent(id)}`);
  }

  async function deleteEmail(id: string) {
    const token = auth.getToken();
    if (!token) return;

    try {
      await adminCommands.deleteCapturedEmail(id, fetch, token);
      toastStore.push({ variant: "success", message: "Email deleted" });
      await pageData.refetch();
    } catch (err) {
      toastStore.push({ variant: "error", message: "Failed to delete email" });
    }
  }
</script>

<section class="emails-page">
  <ListContainer
    title="Captured Emails"
    subtitle="Emails captured during development instead of being sent to real recipients. Only visible in development mode."
    eyebrow="System"
    state={listState}
    loadingMessage="Loading emails..."
    emptyTitle="No captured emails"
    emptyMessage="Emails will appear here when sent in development mode."
    showPagination={false}
  >
    <svelte:fragment slot="filters">
      <form method="GET">
        <FilterToolbar
          ariaLabel="Captured email filters"
          columns={2}
          collapsible
          bind:collapsed={filtersCollapsed}
          summaryText={filterSummaryText}
        >
          <svelte:fragment slot="actions">
            <PoodleIconButton
              icon="refresh-cw"
              variant="secondary"
              size="sm"
              ariaLabel="Refresh emails"
              tooltip="Refresh emails"
              on:click={() => pageData.refetch()}
            />
          </svelte:fragment>

          <PoodleField id="emails-filter-to" label="To address" let:describedBy>
            <PoodleTextInput
              id="emails-filter-to"
              describedBy={describedBy}
              defaultValue={filters.to_address}
              type="email"
              name="to_address"
              placeholder="recipient@example.com"
            />
          </PoodleField>

          <PoodleField id="emails-filter-from" label="From address" let:describedBy>
            <PoodleTextInput
              id="emails-filter-from"
              describedBy={describedBy}
              defaultValue={filters.from_address}
              type="email"
              name="from_address"
              placeholder="sender@example.com"
            />
          </PoodleField>

          <PoodleField id="emails-filter-since" label="Since" let:describedBy>
            <PoodleTextInput
              id="emails-filter-since"
              describedBy={describedBy}
              defaultValue={filters.sinceDate}
              type="date"
              name="since"
            />
          </PoodleField>

          <PoodleField id="emails-filter-until" label="Until" let:describedBy>
            <PoodleTextInput
              id="emails-filter-until"
              describedBy={describedBy}
              defaultValue={filters.untilDate}
              type="date"
              name="until"
            />
          </PoodleField>

          <svelte:fragment slot="secondary">
            <PoodleButton type="submit" variant="primary"><Filter size={16} /> Apply</PoodleButton>
            <PoodleButton type="button" variant="ghost" on:click={() => goto("/system/emails")}>Clear</PoodleButton>
          </svelte:fragment>
        </FilterToolbar>
      </form>
    </svelte:fragment>

    <svelte:fragment slot="error">
      <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
    </svelte:fragment>

    <PoodleGrid columns="repeat(auto-fit, minmax(min(26em, 100%), 1fr))" gap="lg">
      {#each entries as entry}
        {@const accent = entry.wasDelivered ? "#22c55e" : "#3b82f6"}
        {@const href = `/system/emails/${encodeURIComponent(entry.id)}`}
        <PoodleListCard
          href={href}
          title={entry.subject || "(no subject)"}
          subtitle={entry.toAddresses.join(", ")}
          accentColor={accent}
        >
          <svelte:fragment slot="leading">
            <Mail size={30} />
          </svelte:fragment>

          <svelte:fragment slot="actions">
            <CopyActionsMenu
              toastStore={toastStore}
              triggerLabel="Actions"
              copies={[
                {
                  label: "Copy email ID",
                  text: entry.emailId,
                  successMessage: "Copied email ID"
                }
              ]}
              actions={[
                {
                  label: "Open",
                  onSelect: () => openEmail(entry.id)
                },
                {
                  label: "Delete",
                  onSelect: () => deleteEmail(entry.id),
                  destructive: true
                }
              ]}
            />
          </svelte:fragment>

          <span slot="footer">
            From: <strong>{entry.fromAddress}</strong>
          </span>
          <span slot="trailing">
            {new Date(entry.capturedAt).toLocaleString()}
            {#if entry.wasDelivered}
              <span class="delivered-badge">Delivered</span>
            {/if}
          </span>
        </PoodleListCard>
      {/each}
    </PoodleGrid>
  </ListContainer>
</section>

<style>
  .emails-page {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .delivered-badge {
    display: inline-block;
    background: #22c55e;
    color: #000;
    padding: 0.1rem 0.4rem;
    border-radius: 0.2rem;
    font-size: 0.7rem;
    font-weight: 600;
    margin-left: 0.5rem;
  }
</style>
