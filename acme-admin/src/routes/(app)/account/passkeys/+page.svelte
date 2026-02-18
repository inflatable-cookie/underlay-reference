<script lang="ts">
  import { tick } from "svelte";
  import {
    AlertDialog,
    Button,
    Field,
    FormActions,
    FormError,
    TextInput,
    TimeAgo,
    PageLoading
  } from "@decodelabs/underlay/components";
  import { authCommands } from "acme-client";
  import { auth } from "$lib/stores/auth";
  import { useAuthenticatedData } from "@decodelabs/underlay/patterns";
  import { toPublicKeyCreationOptions, credentialCreationToJson } from "@decodelabs/underlay/utils";

  // Passkey type
  type PasskeyCredential = {
    id: string;
    displayName?: string | null;
    metadata: unknown;
    createdAt: string;
    lastUsedAt?: string | null;
  };

  // Page data - fetched when auth is ready
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const passkeyList = await authCommands.listPasskeys(fetch, token).catch(() => []);
      return { passkeys: passkeyList as PasskeyCredential[] };
    },
    {
      defaultValue: { passkeys: [] as PasskeyCredential[] }
    }
  );

  // Passkey state
  type PasskeyStartResponse = {
    options: PublicKeyCredentialCreationOptions;
    stateId: string;
  };

  let passkeyBusy = $state(false);
  let passkeyError = $state<string | null>(null);
  let passkeySuccess = $state<string | null>(null);

  /**
   * Sanitize passkey error messages to remove technical URLs.
   */
  const sanitizePasskeyError = (error: unknown): string => {
    const message = error instanceof Error ? error.message : "Failed to set up passkey.";
    // Remove w3.org spec URLs and other technical details
    return message
      .replace(/\s*See:?\s*https?:\/\/\S+/gi, "")
      .replace(/\s*\(https?:\/\/\S+\)/gi, "")
      .trim();
  };

  let renamePasskeyId = $state<string | null>(null);
  let renamePasskeyValue = $state("");
  let renamePasskeyBusy = $state(false);

  let deletePasskeyId = $state<string | null>(null);
  let deletePasskeyOpen = $state(false);
  let deletePasskeyBusy = $state(false);

  let createPasskeyNameOpen = $state(false);
  let createPasskeyNameValue = $state("");
  let createPasskeyNameInput = $state<HTMLInputElement | null>(null);
  let createPasskeyNameResolver = $state<((value: string | undefined) => void) | null>(null);

  $effect(() => {
    if (!createPasskeyNameOpen) return;
    void tick().then(() => {
      createPasskeyNameInput?.focus();
      createPasskeyNameInput?.select();
    });
  });

  const promptPasskeyName = async (): Promise<string | undefined> => {
    createPasskeyNameValue = "";
    createPasskeyNameOpen = true;
    return new Promise((resolve) => {
      createPasskeyNameResolver = resolve;
    });
  };

  const confirmCreatePasskeyName = () => {
    const value = createPasskeyNameValue.trim();
    createPasskeyNameResolver?.(value || undefined);
    createPasskeyNameResolver = null;
    createPasskeyNameOpen = false;
  };

  const skipCreatePasskeyName = () => {
    createPasskeyNameResolver?.(undefined);
    createPasskeyNameResolver = null;
    createPasskeyNameOpen = false;
  };

  const startRenamePasskey = (pk: PasskeyCredential) => {
    passkeyError = null;
    passkeySuccess = null;
    renamePasskeyId = pk.id;
    renamePasskeyValue = pk.displayName ?? "";
  };

  const cancelRenamePasskey = () => {
    renamePasskeyId = null;
    renamePasskeyValue = "";
  };

  const saveRenamePasskey = async (credentialId: string) => {
    if (renamePasskeyBusy) return;

    const token = auth.getToken();
    if (!token) {
      passkeyError = "Not authenticated";
      return;
    }

    const displayName = renamePasskeyValue.trim();
    if (!displayName) {
      passkeyError = "Passkey name is required.";
      return;
    }

    renamePasskeyBusy = true;

    try {
      await authCommands.renamePasskey(credentialId, { displayName }, fetch, token);
      cancelRenamePasskey();
      passkeySuccess = "Passkey renamed.";
      await pageData.refetch();
    } catch (e) {
      passkeyError = e instanceof Error ? e.message : "Failed to rename passkey.";
    } finally {
      renamePasskeyBusy = false;
    }
  };

  const requestDeletePasskey = (credentialId: string) => {
    passkeyError = null;
    passkeySuccess = null;
    deletePasskeyId = credentialId;
    deletePasskeyOpen = true;
  };

  const confirmDeletePasskey = async () => {
    if (!deletePasskeyId || deletePasskeyBusy) return;

    const token = auth.getToken();
    if (!token) {
      passkeyError = "Not authenticated";
      return;
    }

    deletePasskeyBusy = true;

    try {
      await authCommands.deletePasskey(deletePasskeyId, fetch, token);
      passkeySuccess = "Passkey deleted.";
      deletePasskeyOpen = false;
      deletePasskeyId = null;
      await pageData.refetch();
    } catch (e) {
      passkeyError = e instanceof Error ? e.message : "Failed to delete passkey.";
    } finally {
      deletePasskeyBusy = false;
    }
  };

  const createPasskey = async () => {
    passkeyError = null;
    passkeySuccess = null;

    if (passkeyBusy) return;

    const token = auth.getToken();
    if (!token) {
      passkeyError = "Not authenticated";
      return;
    }

    if (!("PublicKeyCredential" in window) || !navigator.credentials) {
      passkeyError = "Passkeys are not supported in this browser.";
      return;
    }

    passkeyBusy = true;

    try {
      const startData = await authCommands.passkeyConnectStart(fetch, token) as PasskeyStartResponse;
      const publicKey = toPublicKeyCreationOptions(startData.options as any);

      if (!(publicKey as any).challenge) {
        passkeyError = "Passkey setup failed: server did not return a challenge.";
        return;
      }

      const cred = (await navigator.credentials.create({
        publicKey
      })) as PublicKeyCredential | null;

      if (!cred) {
        passkeyError = "Passkey registration was cancelled.";
        return;
      }

      const displayName = await promptPasskeyName();

      await authCommands.passkeyConnectFinish({
        stateId: startData.stateId,
        credential: credentialCreationToJson(cred),
        displayName: displayName?.trim() || undefined
      }, fetch, token);

      passkeySuccess = "Passkey added.";
      await pageData.refetch();
    } catch (e) {
      passkeyError = sanitizePasskeyError(e);
    } finally {
      passkeyBusy = false;
    }
  };
</script>

{#if pageData.loading}
  <PageLoading message="Loading passkeys..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else}

<div class="intro">
  <p>Add a passkey to sign in using your device, without typing your password.</p>
  <p class="hint">Face ID, Touch ID, Windows Hello, etc.</p>
</div>

{#if passkeyError}
  <FormError message={passkeyError} />
{/if}

{#if passkeySuccess}
  <p class="success">{passkeySuccess}</p>
{/if}

{#if !pageData.data?.passkeys?.length}
  <p class="muted">No passkeys added yet.</p>
{:else}
  <ul class="passkey-list">
    {#each pageData.data.passkeys as pk (pk.id)}
      <li class="passkey-item">
        {#if renamePasskeyId === pk.id}
          <Field label="Passkey name">
            <TextInput bind:value={renamePasskeyValue} required />
          </Field>

          <FormActions>
            <Button
              type="button"
              variant="primary"
              onclick={() => saveRenamePasskey(pk.id)}
              disabled={renamePasskeyBusy}
            >
              {renamePasskeyBusy ? "Saving..." : "Save"}
            </Button>
            <Button
              type="button"
              variant="secondary"
              onclick={cancelRenamePasskey}
              disabled={renamePasskeyBusy}
            >
              Cancel
            </Button>
          </FormActions>
        {:else}
          <div class="passkey-item__row">
            <div class="passkey-item__name">{pk.displayName ?? "Unnamed passkey"}</div>
            <div class="passkey-item__actions">
              <Button
                type="button"
                variant="subtle"
                class="button-small"
                onclick={() => startRenamePasskey(pk)}
                disabled={passkeyBusy || deletePasskeyBusy}
              >
                Rename
              </Button>
              <Button
                type="button"
                variant="subtle"
                class="button-small"
                onclick={() => requestDeletePasskey(pk.id)}
                disabled={passkeyBusy || deletePasskeyBusy}
              >
                Delete
              </Button>
            </div>
          </div>

          <div class="passkey-item__meta">
            <span>Created <TimeAgo date={pk.createdAt} /></span>
            {#if pk.lastUsedAt}
              <span>Last used <TimeAgo date={pk.lastUsedAt} /></span>
            {/if}
          </div>
        {/if}
      </li>
    {/each}
  </ul>
{/if}

<FormActions>
  <Button type="button" variant="primary" onclick={createPasskey} disabled={passkeyBusy}>
    {passkeyBusy ? "Working..." : "Add passkey"}
  </Button>
</FormActions>

<AlertDialog
  bind:open={createPasskeyNameOpen}
  showTrigger={false}
  title="Name this passkey"
  description="Give it a name to recognise it later (optional)."
  confirmLabel="Continue"
  cancelLabel="Skip"
  onConfirm={confirmCreatePasskeyName}
  onCancel={skipCreatePasskeyName}
>
  <Field label="Passkey name">
    <TextInput bind:value={createPasskeyNameValue} bind:inputRef={createPasskeyNameInput} />
  </Field>
</AlertDialog>

<AlertDialog
  bind:open={deletePasskeyOpen}
  showTrigger={false}
  title="Delete passkey?"
  description="This removes the passkey from your account. Make sure you still have another way to sign in."
  confirmLabel={deletePasskeyBusy ? "Deleting..." : "Delete passkey"}
  cancelLabel="Cancel"
  onConfirm={confirmDeletePasskey}
  onCancel={() => {
    deletePasskeyOpen = false;
    deletePasskeyId = null;
  }}
/>

{/if}

<style>
  .intro {
    margin-bottom: 1rem;
  }

  .intro p {
    margin: 0;
  }

  .muted {
    color: var(--underlay-color-text-muted, #9ca3af);
  }

  .hint {
    color: var(--underlay-color-text-muted, #9ca3af);
    font-size: 0.85rem;
  }

  .success {
    margin: 0.5rem 0;
    color: #22c55e;
    font-size: 0.9rem;
  }

  .passkey-list {
    margin: 0 0 1rem;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-width: 28rem;
  }

  .passkey-item {
    padding: 0.75rem;
    border-radius: 0.5rem;
    background: rgba(0, 0, 0, 0.15);
    border: 1px solid var(--underlay-color-border-subtle, rgba(255, 255, 255, 0.1));
  }

  .passkey-item__row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .passkey-item__actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .passkey-item__actions :global(.button-small) {
    font-size: 0.8rem;
    padding: 0.4em 0.8em;
  }

  .passkey-item__name {
    font-weight: 600;
  }

  .passkey-item__meta {
    margin-top: 0.25rem;
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
    color: var(--underlay-color-text-muted, #9ca3af);
    font-size: 0.85rem;
  }
</style>
