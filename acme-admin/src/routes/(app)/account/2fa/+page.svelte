<script lang="ts">
import {
  useAuthenticatedData
} from "@decodelabs/underlay/runtime/auth";
import {
  CodeInput as PoodleCodeInput,
  AlertDialog as PoodleAlertDialog,
  Button as PoodleButton,
  Callout as PoodleCallout,
  Card as PoodleCard,
  FormActions as PoodleFormActions
  } from "@poodle/svelte";
  import { PageLoading } from "@poodle/svelte";
  import { authCommands } from "@api-client";
  import { auth } from "$lib/stores/auth";
  
  // Page data - fetched when auth is ready
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const totpStatus = await authCommands.totpStatus(fetch, token).catch(() => ({ enabled: false }));
      return { totpEnabled: totpStatus.enabled };
    },
    {
      defaultValue: { totpEnabled: false },
      onSuccess: (data) => {
        // Auto-trigger TOTP setup if not enabled
        if (!data.totpEnabled && !totpSetup) {
          setupTotp();
        }
      }
    }
  );

  // TOTP state
  let totpError = $state<string | null>(null);
  let totpBusy = $state(false);

  let totpSetup = $state<
    | {
        setupId: string;
        otpauthUri: string;
        qrSvg: string;
        backupCodes: string[];
      }
    | undefined
  >(undefined);

  let disableTotpOpen = $state(false);
  let enableCode = $state("");

  const setupTotp = async () => {
    if (totpBusy) return;

    const token = auth.getToken();
    if (!token) {
      totpError = "Not authenticated";
      return;
    }

    totpBusy = true;
    totpError = null;

    try {
      const setup = await authCommands.totpSetup(fetch, token);
      totpSetup = setup as typeof totpSetup;
    } catch (e) {
      totpError = e instanceof Error ? e.message : "Failed to start 2FA setup";
    } finally {
      totpBusy = false;
    }
  };

  const enableTotp = async () => {
    if (totpBusy || !totpSetup) return;

    const token = auth.getToken();
    if (!token) {
      totpError = "Not authenticated";
      return;
    }

    const code = enableCode.trim();
    if (!code) {
      totpError = "Enter the code from your authenticator app";
      return;
    }

    totpBusy = true;
    totpError = null;

    try {
      await authCommands.totpEnable({ setupId: totpSetup.setupId, code }, fetch, token);
      totpSetup = undefined;
      enableCode = "";
      await pageData.refetch();
    } catch (e) {
      totpError = e instanceof Error ? e.message : "Failed to enable 2FA";
    } finally {
      totpBusy = false;
    }
  };

  const disableTotp = async () => {
    if (totpBusy) return;

    const token = auth.getToken();
    if (!token) {
      totpError = "Not authenticated";
      return;
    }

    totpBusy = true;
    totpError = null;

    try {
      await authCommands.totpDisable(fetch, token);
      disableTotpOpen = false;
      await pageData.refetch();
      // Re-trigger setup after disabling
      setupTotp();
    } catch (e) {
      totpError = e instanceof Error ? e.message : "Failed to disable 2FA";
    } finally {
      totpBusy = false;
    }
  };

  /**
   * Validate SVG content for QR code rendering.
   */
  const validateQrSvg = (svg: string): boolean => {
    if (!svg) return false;
    const trimmed = svg.trim();
    if (!trimmed.includes("<svg") || !trimmed.endsWith("</svg>")) return false;
    const dangerous = /<script|javascript:|\bon\w+\s*=/i;
    if (dangerous.test(svg)) return false;
    return true;
  };
</script>

{#if pageData.loading}
  <PageLoading presentation="inline" message="Loading 2FA settings..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else}

{#if totpError}
  <PoodleCallout tone="danger" message={totpError} announceMode="polite" />
{/if}

{#if pageData.data?.totpEnabled}
  <PoodleCard>
    <p class="success-message">Two-factor authentication is enabled on your account.</p>
    <p class="muted">Your account is protected with an authenticator app.</p>

    <PoodleFormActions align="end">
      <PoodleButton
        type="button"
        variant="secondary"
        disabled={totpBusy}
        onClick={() => (disableTotpOpen = true)}
      >
        Disable 2FA
      </PoodleButton>
    </PoodleFormActions>
  </PoodleCard>

  <PoodleAlertDialog
    bind:open={disableTotpOpen}
    title="Disable 2FA?"
    description="This removes TOTP protection from your account."
    confirmLabel={totpBusy ? "Disabling..." : "Disable 2FA"}
    cancelLabel="Cancel"
    onConfirm={disableTotp}
    onCancel={() => (disableTotpOpen = false)}
    tone="danger"
  />
{:else if totpBusy && !totpSetup}
  <div class="intro">
    <p class="muted">Setting up two-factor authentication...</p>
  </div>
{:else if totpSetup}
  <div class="intro">
    <p>Scan this QR code with your authenticator app.</p>
    <p class="hint">Google Authenticator, Authy, 1Password, etc.</p>
  </div>

  <PoodleCard>
    <div class="totp-setup">
      <div class="totp-setup__qr" aria-label="TOTP QR code">
        {#if validateQrSvg(totpSetup.qrSvg)}
          {@html totpSetup.qrSvg}
        {:else}
          <p class="totp-setup__qr-error">Unable to display QR code. Use manual setup below.</p>
        {/if}
      </div>

      <div class="totp-setup__details">
        <details>
          <summary>Show manual setup URI</summary>
          <pre class="totp-setup__uri">{totpSetup.otpauthUri}</pre>
        </details>

        <details>
          <summary>Show backup codes</summary>
          <p class="totp-setup__hint">
            Save these somewhere safe. Each can be used once.
          </p>
          <pre class="totp-setup__codes">
{totpSetup.backupCodes.join("\n")}
          </pre>
        </details>

        <PoodleCodeInput
          value={enableCode}
          label="Enter code from your authenticator app"
          onValueChange={(value) => { enableCode = value; }}
          onComplete={enableTotp}
        />

        <PoodleFormActions align="end">
          <PoodleButton type="button" variant="primary" disabled={totpBusy} onClick={enableTotp}>
            {totpBusy ? "Enabling..." : "Enable 2FA"}
          </PoodleButton>
        </PoodleFormActions>
      </div>
    </div>
  </PoodleCard>
{:else}
  <div class="intro">
    <p class="muted">Two-factor authentication is not enabled.</p>
  </div>
  <PoodleFormActions align="end">
    <PoodleButton type="button" variant="primary" disabled={totpBusy} onClick={setupTotp}>
      {totpBusy ? "Loading..." : "Set up 2FA"}
    </PoodleButton>
  </PoodleFormActions>
{/if}

{/if}

<style>
  .intro {
    margin-bottom: 1rem;
  }

  .intro p {
    margin: 0 0 0.5rem;
  }

  .intro p:last-child {
    margin-bottom: 0;
  }

  .muted {
    color: var(--underlay-color-text-muted, #9ca3af);
  }

  .hint {
    color: var(--underlay-color-text-muted, #9ca3af);
    font-size: 0.85rem;
  }

  .success-message {
    color: #22c55e;
  }

  .totp-setup {
    display: grid;
    grid-template-columns: 260px minmax(0, 1fr);
    gap: 1.25rem;
    align-items: start;
  }

  .totp-setup__qr {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    border-radius: 1rem;
    border: 1px solid var(--underlay-color-border-subtle, rgba(255, 255, 255, 0.1));
    background: var(--underlay-color-surface-subtle, rgba(255, 255, 255, 0.03));
  }

  .totp-setup__qr :global(svg) {
    width: 220px;
    height: 220px;
  }

  .totp-setup__qr-error {
    margin: 0;
    padding: 1rem;
    color: var(--underlay-color-text-muted, #9ca3af);
    font-size: 0.9rem;
    text-align: center;
  }

  .totp-setup__details {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .totp-setup__uri,
  .totp-setup__codes {
    margin: 0.6rem 0 0;
    padding: 0.75rem;
    border-radius: 0.75rem;
    background: rgba(15, 23, 42, 0.55);
    border: 1px solid var(--underlay-color-border-subtle, rgba(255, 255, 255, 0.1));
    overflow: auto;
    font-size: 0.85rem;
  }

  .totp-setup__hint {
    margin: 0.5rem 0 0;
    color: var(--underlay-color-text-muted, #9ca3af);
    font-size: 0.85rem;
  }

  @media (max-width: 700px) {
    .totp-setup {
      grid-template-columns: 1fr;
    }
  }
</style>
