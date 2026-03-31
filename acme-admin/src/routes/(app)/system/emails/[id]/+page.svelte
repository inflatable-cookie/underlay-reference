<script lang="ts">
import {
  useToasts,
  useAuthenticatedData
} from "@decodelabs/underlay/runtime";
import {
  AlertDialog as PoodleAlertDialog,
  Button as PoodleButton,
  Callout as PoodleCallout,
  Card as PoodleCard,
  Code as PoodleCode,
  DetailItem as PoodleDetailItem,
  MetaBar as PoodleMetaBar,
  MetaItem as PoodleMetaItem,
  Menu as PoodleMenu,
  Tabs,
  type MenuItem,
  type TabItem
  } from "@poodle/svelte-primitives";
  import { DetailSection as PoodleDetailSection,
  PageHeader as PoodlePageHeader,
  PageLoading } from "@poodle/svelte-composites";
import { goto } from "$app/navigation";
import { page } from "$app/stores";
    import { Pill as PoodlePill } from "@poodle/svelte-primitives";
  import { adminCommands } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { copyToClipboard } from "@decodelabs/underlay/runtime";
  import type { CapturedEmailDetail } from "@api-client";

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
  let showDeleteConfirm = $state(false);
  const actionItems: MenuItem[] = [
    { value: "delete", label: "Delete", tone: "danger" },
    { value: "separator-copy", label: "", kind: "separator" },
    { value: "copy-id", label: "Copy ID" },
    { value: "copy-from", label: "Copy from address" },
    { value: "copy-to", label: "Copy to addresses" }
  ];

  function getStatusLabel() {
    if (!email) return "Captured";
    if (email.wasDelivered) return "Delivered";
    if (email.deliveryError) return "Failed";
    return "Captured";
  }

  function getStatusTone() {
    if (!email) return "neutral";
    if (email.wasDelivered) return "success";
    if (email.deliveryError) return "danger";
    return "neutral";
  }

  // Build dynamic tabs based on email content
  const emailTabs = $derived.by(() => {
    if (!email) return [];
    const tabs: TabItem[] = [];
    if (email.htmlBody) tabs.push({ value: "html", label: "HTML Preview" });
    if (email.textBody) tabs.push({ value: "text", label: "Plain Text" });
    if (email.htmlBody) tabs.push({ value: "source", label: "HTML Source" });
    if (email.headersJson && Object.keys(email.headersJson).length > 0) {
      tabs.push({ value: "headers", label: "Headers" });
    }
    return tabs;
  });

  async function handleAction(value: string) {
    if (!email) return;

    if (value === "delete") {
      showDeleteConfirm = true;
      return;
    }

    if (value === "copy-id") {
      await copyToClipboard(toastStore, email.emailId, "Copied email ID");
      return;
    }

    if (value === "copy-from") {
      await copyToClipboard(toastStore, email.fromAddress, "Copied from address");
      return;
    }

    if (value === "copy-to") {
      await copyToClipboard(toastStore, email.toAddresses.join(", "), "Copied to addresses");
    }
  }
</script>

{#if pageData.loading}
  <PageLoading presentation="inline" message="Loading email..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if email}
  <section class="email-detail">
    <div class="email-detail__header">
      <PoodlePageHeader
        section="Captured Email"
        title={email.subject || "(no subject)"}
        backHref="/system/emails"
        backLabel="Back to emails"
      >
        {#snippet actions()}
          <PoodleMenu
            items={actionItems}
            placement="bottom-end"
            triggerAriaLabel="Email actions"
            on:action={(event) => void handleAction(event.detail.value)}
          >
            <svelte:fragment slot="trigger">
              <PoodleButton variant="secondary">Actions</PoodleButton>
            </svelte:fragment>
          </PoodleMenu>
        {/snippet}
      </PoodlePageHeader>

      <div class="email-detail__meta">
      <PoodleMetaBar ariaLabel="Email metadata">
        <PoodleMetaItem label="ID">
          <PoodleCode inline source={email.emailId} showCopyButton />
        </PoodleMetaItem>
        <PoodlePill tone={getStatusTone()} appearance="badge" size="lg">
          {getStatusLabel()}
        </PoodlePill>
      </PoodleMetaBar>
      </div>
    </div>

    <Tabs bind:value={activeTab} items={emailTabs} variant="card" size="sm" ariaLabel="Email detail sections" let:activeValue>
      {#if activeValue === "html" && email.htmlBody}
        <PoodleCard>
          <div class="detail-card-grid">
            <PoodleDetailSection title="Addresses" columns={2} separated={false}>
              <PoodleDetailItem presentation="surface" label="From" value={email.fromAddress} />
              <PoodleDetailItem presentation="surface" label="To" value={email.toAddresses.join(", ")} />
              {#if email.replyTo}
                <PoodleDetailItem presentation="surface" label="Reply-To" value={email.replyTo} />
              {/if}
              {#if email.ccAddresses.length > 0}
                <PoodleDetailItem presentation="surface" label="CC" value={email.ccAddresses.join(", ")} />
              {/if}
              {#if email.bccAddresses.length > 0}
                <PoodleDetailItem presentation="surface" label="BCC" value={email.bccAddresses.join(", ")} />
              {/if}
            </PoodleDetailSection>
            <PoodleDetailSection title="Metadata" columns={2} separated={false}>
              <PoodleDetailItem presentation="surface" label="Captured" value={new Date(email.capturedAt).toLocaleString()} />
            </PoodleDetailSection>
          </div>
        </PoodleCard>

        <div class="email-detail__body">
          <div class="email-detail__preview">
            <iframe
              title="Email HTML preview"
              srcdoc={email.htmlBody}
              sandbox="allow-same-origin"
            ></iframe>
          </div>
        </div>
      {:else if activeValue === "text" && email.textBody}
        <div class="email-detail__body">
          <pre class="email-detail__text">{email.textBody}</pre>
        </div>
      {:else if activeValue === "source" && email.htmlBody}
        <div class="email-detail__body">
          <pre class="email-detail__source">{email.htmlBody}</pre>
        </div>
      {:else if activeValue === "headers" && email.headersJson}
        <div class="email-detail__body">
          <pre class="email-detail__headers">{JSON.stringify(email.headersJson, null, 2)}</pre>
        </div>
      {/if}
    </Tabs>
  </section>

  <PoodleAlertDialog
    open={showDeleteConfirm}
    title="Delete email"
    description="Are you sure you want to delete this captured email? This action cannot be undone."
    confirmLabel="Delete"
    tone="danger"
    onConfirm={async () => {
      const token = auth.getToken();
      if (!token) throw new Error("Not authenticated");
      await adminCommands.deleteCapturedEmail(email.id, fetch, token);
      await goto("/system/emails");
    }}
    onCancel={() => {
      showDeleteConfirm = false;
    }}
  >
    <p><strong>{email.subject || "(no subject)"}</strong></p>
  </PoodleAlertDialog>
{/if}

<style>
  .email-detail {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .email-detail__header {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .email-detail__meta {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .detail-card-grid {
    display: grid;
    gap: 1rem;
  }

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
