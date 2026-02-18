<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { CopyActionsMenu, PageHeader, useToasts, useAuthenticatedData } from "@decodelabs/underlay/patterns";
  import { FilterBar } from "@decodelabs/underlay/patterns";
  import {
    Button,
    EmptyState,
    Field,
    FormError,
    ListCard,
    ListGrid,
    PageLoading,
    TextInput
  } from "@decodelabs/underlay/components";
  import Mail from "lucide-svelte/icons/mail";
  import Filter from "lucide-svelte/icons/filter";
  import { adminCommands } from "acme-client";
  import { auth } from "$lib/stores/auth";
  import type { CapturedEmailSummary } from "acme-client";

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
  <PageHeader section="Captured Emails" backHref="/system" backLabel="Back to system">
    Emails captured during development instead of being sent to real recipients.
    Only visible in development mode.
  </PageHeader>

  <FilterBar title="Filters" startCollapsed={true} onRefresh={() => pageData.refetch()}>
    <form class="emails-page__filters" method="GET">
      <div class="emails-page__filters-row">
        <Field label="To address">
          <TextInput
            type="email"
            name="to_address"
            value={filters.to_address}
            placeholder="recipient@example.com"
          />
        </Field>
        <Field label="From address">
          <TextInput
            type="email"
            name="from_address"
            value={filters.from_address}
            placeholder="sender@example.com"
          />
        </Field>
      </div>
      <div class="emails-page__filters-row">
        <Field label="Since">
          <TextInput
            type="date"
            name="since"
            value={filters.sinceDate}
          />
        </Field>
        <Field label="Until">
          <TextInput
            type="date"
            name="until"
            value={filters.untilDate}
          />
        </Field>
        <div class="emails-page__filters-actions">
          <Button type="submit" variant="primary"><Filter size={16} /> Apply</Button>
          <a href="/system/emails">Clear</a>
        </div>
      </div>
    </form>
  </FilterBar>

  {#if pageData.loading}
    <PageLoading message="Loading emails..." />
  {:else if pageData.error}
    <FormError message={pageData.error} />
  {:else if entries.length === 0}
    <EmptyState title="No captured emails" description="Emails will appear here when sent in development mode." variant="compact" />
  {:else}
    <ListGrid minItemWidth={26}>
      {#each entries as entry}
        {@const accent = entry.wasDelivered ? "#22c55e" : "#3b82f6"}
        {@const href = `/system/emails/${encodeURIComponent(entry.id)}`}
        <ListCard
          href={href}
          title={entry.subject || "(no subject)"}
          subtitle={entry.toAddresses.join(", ")}
          accent={accent}
        >
          {#snippet media()}
            <Mail size={30} />
          {/snippet}

          {#snippet actions({ trigger, align })}
            <CopyActionsMenu
              toastStore={toastStore}
              {trigger}
              {align}
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
          {/snippet}

          <span>
            From: <strong>{entry.fromAddress}</strong>
          </span>
          <span>
            {new Date(entry.capturedAt).toLocaleString()}
            {#if entry.wasDelivered}
              <span class="delivered-badge">Delivered</span>
            {/if}
          </span>
        </ListCard>
      {/each}
    </ListGrid>
  {/if}
</section>

<style>
  .emails-page {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .emails-page__filters {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .emails-page__filters-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
  }

  :global(.emails-page__filters label) {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
    flex: 1 1 180px;
    font-size: 0.8rem;
  }

  :global(.emails-page__filters input) {
    border-radius: 0.35rem;
    border: none;
    padding: 0.45rem 0.55rem;
    background: rgba(15, 23, 42, 0.9);
    color: inherit;
  }

  :global(.emails-page__filters input::placeholder) {
    color: rgba(148, 163, 184, 0.7);
  }

  .emails-page__filters-actions {
    display: flex;
    align-items: flex-end;
    gap: 0.5rem;
    margin-left: auto;
  }

  .emails-page__filters-actions a {
    font-size: 0.8rem;
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
