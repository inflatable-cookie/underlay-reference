<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import {
    PageHeader,
    PageHeaderMeta,
    PageHeaderMetaRow,
    PageHeaderMetaItem,
    PageHeaderMetaSeparator,
    useToasts,
    useAuthenticatedData
  } from "@decodelabs/underlay/patterns";
  import {
    AlertDialog,
    Code,
    DetailsCard,
    DetailsItem,
    DetailsSection,
    DropdownMenu,
    FormError,
    PageLoading,
    TabsRoot,
    TabsList,
    TabsTrigger,
    TabsContent,
    Pill
  } from "@decodelabs/underlay/components";
  import Copy from "lucide-svelte/icons/copy";
  import { adminCommands } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import type { CapturedEmailDetail } from "acme-client";

  const toastStore = useToasts();
  let confirmDeleteOpen = $state(false);

  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const id = $page.params.id;
      if (!id) throw new Error("Email ID required");
      const email = await adminCommands.getCapturedEmail(id, fetch, token);
      return { email };
    },
    {
      getToken: () => auth.getToken()
    }
  );

  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  const email = $derived(pageData.data?.email as CapturedEmailDetail | undefined);

  let activeTab = $state("html");

  async function deleteEmail() {
    const token = auth.getToken();
    if (!token || !email) return;

    try {
      await adminCommands.deleteCapturedEmail(email.id, fetch, token);
      toastStore.push({ variant: "success", message: "Email deleted" });
      await goto("/system/emails");
    } catch {
      toastStore.push({ variant: "error", message: "Failed to delete email" });
    }
  }

  function copyToClipboard(text: string, label: string) {
    navigator.clipboard.writeText(text);
    toastStore.push({ variant: "success", message: `Copied ${label}` });
  }

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

  const menuItems = $derived(email ? [
    { label: "Delete email", onSelect: () => { confirmDeleteOpen = true; }, destructive: true }
  ] : []);
</script>

{#if pageData.loading}
  <PageLoading message="Loading email..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if email}
  <AlertDialog
    bind:open={confirmDeleteOpen}
    title="Delete email"
    description="Are you sure you want to delete this captured email? This action cannot be undone."
    confirmLabel="Delete"
    showTrigger={false}
    onConfirm={deleteEmail}
  />

  <section class="email-detail">
    <PageHeader
      section="Captured Email"
      title={email.subject || "(no subject)"}
      backHref="/system/emails"
      backLabel="Back to emails"
    >
      {#snippet actions()}
        <DropdownMenu items={menuItems} triggerAriaLabel="Email actions" />
      {/snippet}

      <PageHeaderMeta>
        <PageHeaderMetaRow>
          <PageHeaderMetaItem label="ID">
            <Code copy>{email.emailId}</Code>
          </PageHeaderMetaItem>
          <PageHeaderMetaSeparator />
          <Pill accent={getStatusAccent()}>{getStatusLabel()}</Pill>
        </PageHeaderMetaRow>
      </PageHeaderMeta>
    </PageHeader>

    <DetailsCard>
      <DetailsSection legend="Addresses">
        <DetailsItem label="From">
          <span class="email-detail__value">
            {email.fromAddress}
            <button
              type="button"
              class="copy-btn"
              onclick={() => copyToClipboard(email.fromAddress, "from address")}
            >
              <Copy size={14} />
            </button>
          </span>
        </DetailsItem>
        <DetailsItem label="To">
          <span class="email-detail__value">
            {email.toAddresses.join(", ")}
            <button
              type="button"
              class="copy-btn"
              onclick={() => copyToClipboard(email.toAddresses.join(", "), "to addresses")}
            >
              <Copy size={14} />
            </button>
          </span>
        </DetailsItem>
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

    <TabsRoot class="email-detail__tabs" bind:value={activeTab}>
      <TabsList>
        {#if email.htmlBody}
          <TabsTrigger value="html">HTML Preview</TabsTrigger>
        {/if}
        {#if email.textBody}
          <TabsTrigger value="text">Plain Text</TabsTrigger>
        {/if}
        {#if email.htmlBody}
          <TabsTrigger value="source">HTML Source</TabsTrigger>
        {/if}
        {#if email.headersJson && Object.keys(email.headersJson).length > 0}
          <TabsTrigger value="headers">Headers</TabsTrigger>
        {/if}
      </TabsList>

      <div class="email-detail__body">
        {#if email.htmlBody}
          <TabsContent value="html">
            <div class="email-detail__preview">
              <iframe
                title="Email HTML preview"
                srcdoc={email.htmlBody}
                sandbox="allow-same-origin"
              ></iframe>
            </div>
          </TabsContent>
        {/if}

        {#if email.textBody}
          <TabsContent value="text">
            <pre class="email-detail__text">{email.textBody}</pre>
          </TabsContent>
        {/if}

        {#if email.htmlBody}
          <TabsContent value="source">
            <pre class="email-detail__source">{email.htmlBody}</pre>
          </TabsContent>
        {/if}

        {#if email.headersJson && Object.keys(email.headersJson).length > 0}
          <TabsContent value="headers">
            <pre class="email-detail__headers">{JSON.stringify(email.headersJson, null, 2)}</pre>
          </TabsContent>
        {/if}
      </div>
    </TabsRoot>
  </section>
{/if}

<style>
  .email-detail {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .email-detail__value {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    word-break: break-word;
  }

  .copy-btn {
    background: transparent;
    border: none;
    color: var(--admin-color-text-muted);
    cursor: pointer;
    padding: 0.2rem;
    border-radius: 0.2rem;
    display: inline-flex;
    align-items: center;
  }

  .copy-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--admin-color-text);
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

  :global(.email-detail__body .underlay-tabs-content) {
    padding-top: 0;
  }

  :global(.email-detail__tabs .underlay-tabs-content) {
    margin-top: 0;
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
