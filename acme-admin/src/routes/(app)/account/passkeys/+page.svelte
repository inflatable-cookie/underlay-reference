<script lang="ts">
import {
  useAuthenticatedData
} from "@decodelabs/underlay/runtime/auth";
import {
  AlertDialog as PoodleAlertDialog,
  Button as PoodleButton,
  Callout as PoodleCallout,
  Field as PoodleField,
  FormActions as PoodleFormActions,
  TextInput as PoodleTextInput,
  TimeAgo
  } from "@poodle/svelte";
  import { tick } from "svelte";
  import { PageLoading } from "@poodle/svelte";
  import { authCommands } from "@api-client";
  import { auth } from "$lib/stores/auth";
    import { toPublicKeyCreationOptions, credentialCreationToJson } from "@decodelabs/underlay/utils/webauthn";

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
  let createPasskeyNameResolver = $state<((value: string | undefined) => void) | null>(null);

  $effect(() => {
    if (!createPasskeyNameOpen) return;
    void tick().then(() => {
      const input = document.getElementById("create-passkey-name") as HTMLInputElement | null;
      input?.focus();
      input?.select();
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

  function validationState(error?: string | null) {
    return error ? "invalid" : "none";
  }

  function renameFieldError(passkeyId: string): string | null {
    if (renamePasskeyId !== passkeyId) return null;
    return passkeyError === "Passkey name is required." ? passkeyError : null;
  }
</script>

{#if pageData.loading}
  <PageLoading presentation="inline" message="Loading passkeys..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else}

<div class="intro">
  <p>Add a passkey to sign in using your device, without typing your password.</p>
  <p class="hint">Face ID, Touch ID, Windows Hello, etc.</p>
</div>

{#if passkeyError}
  <PoodleCallout tone="danger" message={passkeyError} announceMode="polite" />
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
          <PoodleField
            id={`rename-passkey-${pk.id}`}
            label="Passkey name"
            error={renameFieldError(pk.id)}
            validationState={validationState(renameFieldError(pk.id))}
            required
          >
            {#snippet control({ describedBy, validationState })}
              <PoodleTextInput
                id={`rename-passkey-${pk.id}`}
                value={renamePasskeyValue}
                describedBy={describedBy}
                validationState={validationState}
                maxLength={120}
                onValueChange={(nextValue) => { renamePasskeyValue = nextValue; }}
                onSubmit={() => saveRenamePasskey(pk.id)}
              />
            {/snippet}
          </PoodleField>

          <PoodleFormActions align="end">
            <PoodleButton
              type="button"
              variant="primary"
              disabled={renamePasskeyBusy}
              onClick={() => saveRenamePasskey(pk.id)}
            >
              {renamePasskeyBusy ? "Saving..." : "Save"}
            </PoodleButton>
            <PoodleButton
              type="button"
              variant="secondary"
              disabled={renamePasskeyBusy}
              onClick={cancelRenamePasskey}
            >
              Cancel
            </PoodleButton>
          </PoodleFormActions>
        {:else}
          <div class="passkey-item__row">
            <div class="passkey-item__name">{pk.displayName ?? "Unnamed passkey"}</div>
            <div class="passkey-item__actions">
              <PoodleButton
                type="button"
                variant="ghost"
                size="sm"
                className="button-small"
                disabled={passkeyBusy || deletePasskeyBusy}
                onClick={() => startRenamePasskey(pk)}
              >
                Rename
              </PoodleButton>
              <PoodleButton
                type="button"
                variant="ghost"
                tone="danger"
                size="sm"
                className="button-small"
                disabled={passkeyBusy || deletePasskeyBusy}
                onClick={() => requestDeletePasskey(pk.id)}
              >
                Delete
              </PoodleButton>
            </div>
          </div>

          <div class="passkey-item__meta">
            <span>Created <TimeAgo datetime={pk.createdAt} /></span>
            {#if pk.lastUsedAt}
              <span>Last used <TimeAgo datetime={pk.lastUsedAt} /></span>
            {/if}
          </div>
        {/if}
      </li>
    {/each}
  </ul>
{/if}

<PoodleFormActions align="end">
  <PoodleButton type="button" variant="primary" disabled={passkeyBusy} onClick={createPasskey}>
    {passkeyBusy ? "Working..." : "Add passkey"}
  </PoodleButton>
</PoodleFormActions>

<PoodleAlertDialog
  bind:open={createPasskeyNameOpen}
  title="Name this passkey"
  description="Give it a name to recognise it later (optional)."
  confirmLabel="Continue"
  cancelLabel="Skip"
  onConfirm={confirmCreatePasskeyName}
  onCancel={skipCreatePasskeyName}
  tone="warning"
>
  <PoodleField id="create-passkey-name" label="Passkey name">
    {#snippet control({ describedBy })}
      <PoodleTextInput
        id="create-passkey-name"
        value={createPasskeyNameValue}
        describedBy={describedBy}
        maxLength={120}
        onValueChange={(nextValue) => { createPasskeyNameValue = nextValue; }}
        onSubmit={confirmCreatePasskeyName}
      />
    {/snippet}
  </PoodleField>
</PoodleAlertDialog>

<PoodleAlertDialog
  bind:open={deletePasskeyOpen}
  title="Delete passkey?"
  description="This removes the passkey from your account. Make sure you still have another way to sign in."
  confirmLabel={deletePasskeyBusy ? "Deleting..." : "Delete passkey"}
  cancelLabel="Cancel"
  onConfirm={confirmDeletePasskey}
  onCancel={() => {
    deletePasskeyOpen = false;
    deletePasskeyId = null;
  }}
  tone="danger"
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
