<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import {
    DetailPageShell,
    DetailMeta,
    DetailMetaId,
    DetailMetaSeparator,
    EntityActionsMenu,
    useToasts,
    useAuthenticatedData
  } from "@decodelabs/underlay/patterns";
  import {
    DetailsCard,
    DetailsItem,
    DetailsSection,
    FormError,
    PageLoading,
    Pill
  } from "@decodelabs/underlay/components";
  import { adminCommands } from "acme-client";
  import { auth } from "$lib/stores/auth";
  import type { CapturedEmailDetail } from "acme-client";

  const toastStore = useToasts();

  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const id = $page.params.id;
      if (!id) throw new Error("Email ID required");
      const email = await adminCommands.getCapturedEmail(id, fetch, token);
      return { email };
    },
    {}
  );

  const email = $derived(pageData.data?.email as CapturedEmailDetail | undefined);

  let activeTab = $state("html");

  function getStatusLabel() {
    if (!email) return "Captured";
    if (email.wasDelivered) return "Delivered";
    if (email.deliveryError) return "Failed";
    return "Captured";
  }

  function getStatusAccent() {
    if (!email) return "#3b82f6";
    if (email.wasDelivered) return "#10b981";
    if (email.deliveryError) return "#ef4444";
    return "#3b82f6";
  }

  // Build dynamic tabs based on email content
  const emailTabs = $derived.by(() => {
    if (!email) return [];
    const tabs: { value: string; label: string }[] = [];
    if (email.htmlBody) tabs.push({ value: "html", label: "HTML Preview" });
    if (email.textBody) tabs.push({ value: "text", label: "Plain Text" });
    if (email.htmlBody) tabs.push({ value: "source", label: "HTML Source" });
    if (email.headersJson && Object.keys(email.headersJson).length > 0) {
      tabs.push({ value: "headers", label: "Headers" });
    }
    return tabs;
  });
</script>

{#if pageData.loading}
  <PageLoading message="Loading email..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if email}
  <DetailPageShell
    section="Captured Email"
    title={email.subject || "(no subject)"}
    backHref="/system/emails"
    backLabel="Back to emails"
    tabs={emailTabs}
    bind:activeTab
  >
    {#snippet meta()}
      <DetailMeta>
        <DetailMetaId value={email.emailId} />
        <DetailMetaSeparator />
        <Pill accent={getStatusAccent()}>{getStatusLabel()}</Pill>
      </DetailMeta>
    {/snippet}

    {#snippet actions()}
      <EntityActionsMenu
        toastStore={toastStore}
        copies={[
          { label: "Copy ID", text: email.emailId, successMessage: "Copied email ID" },
          { label: "Copy from address", text: email.fromAddress, successMessage: "Copied from address" },
          { label: "Copy to addresses", text: email.toAddresses.join(", "), successMessage: "Copied to addresses" }
        ]}
        deleteConfig={{
          entityLabel: email.subject || "(no subject)",
          title: "Delete email",
          description: "Are you sure you want to delete this captured email? This action cannot be undone.",
          confirmLabel: "Delete",
          execute: async () => {
            const token = auth.getToken();
            if (!token) throw new Error("Not authenticated");
            await adminCommands.deleteCapturedEmail(email.id, fetch, token);
          }
        }}
        onDeleteSuccess={() => goto("/system/emails")}
      />
    {/snippet}

    {#snippet tabContent(tab)}
      {#if tab === "html" && email.htmlBody}
        <DetailsCard>
          <DetailsSection legend="Addresses">
            <DetailsItem label="From" value={email.fromAddress} />
            <DetailsItem label="To" value={email.toAddresses.join(", ")} />
            {#if email.replyTo}
              <DetailsItem label="Reply-To" value={email.replyTo} />
            {/if}
            {#if email.ccAddresses.length > 0}
              <DetailsItem label="CC" value={email.ccAddresses.join(", ")} />
            {/if}
            {#if email.bccAddresses.length > 0}
              <DetailsItem label="BCC" value={email.bccAddresses.join(", ")} />
            {/if}
          </DetailsSection>
          <DetailsSection legend="Metadata">
            <DetailsItem label="Captured" value={new Date(email.capturedAt).toLocaleString()} />
          </DetailsSection>
        </DetailsCard>

        <div class="email-detail__body">
          <div class="email-detail__preview">
            <iframe
              title="Email HTML preview"
              srcdoc={email.htmlBody}
              sandbox="allow-same-origin"
            ></iframe>
          </div>
        </div>
      {:else if tab === "text" && email.textBody}
        <div class="email-detail__body">
          <pre class="email-detail__text">{email.textBody}</pre>
        </div>
      {:else if tab === "source" && email.htmlBody}
        <div class="email-detail__body">
          <pre class="email-detail__source">{email.htmlBody}</pre>
        </div>
      {:else if tab === "headers" && email.headersJson}
        <div class="email-detail__body">
          <pre class="email-detail__headers">{JSON.stringify(email.headersJson, null, 2)}</pre>
        </div>
      {/if}
    {/snippet}
  </DetailPageShell>
{/if}

<style>
  .email-detail__body {
    background: var(--admin-color-surface-subtle);
    border: 1px solid var(--admin-color-border-subtle);
    border-radius: 0.5rem;
    overflow: hidden;
    margin-top: 0.75rem;
  }

  .email-detail__preview {
    height: 600px;
    background: #fff;
  }

  .email-detail__preview iframe {
    width: 100%;
    height: 100%;
    border: none;
  }

  .email-detail__text,
  .email-detail__source,
  .email-detail__headers {
    margin: 0;
    padding: 1rem;
    font-size: 0.85rem;
    line-height: 1.5;
    overflow-x: auto;
    white-space: pre-wrap;
    word-wrap: break-word;
  }
</style>
