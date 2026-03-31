<script lang="ts">
import {
  PasswordRequirements
} from "@decodelabs/underlay/patterns";
import {
  useAuthenticatedData
} from "@decodelabs/underlay/runtime";
import {
  Callout as PoodleCallout,
  CodeInput as PoodleCodeInput
} from "@poodle/svelte-primitives";
import { PageLoading } from "@poodle/svelte-composites";
import {
  Button as PoodleButton,
  Card as PoodleCard,
  Field as PoodleField,
  FormActions as PoodleFormActions,
  TextInput as PoodleTextInput
} from "@poodle/svelte-primitives";
import { authCommands } from "@api-client";
import { auth } from "$lib/stores/auth";

  // Page data - fetched when auth is ready
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const totpStatus = await authCommands.totpStatus(fetch, token).catch(() => ({ enabled: false }));
      return { totpEnabled: totpStatus.enabled };
    },
    {
      defaultValue: { totpEnabled: false }
    }
  );

  // State machine: "verify" | "password" | "success"
  type PasswordStep = "verify" | "password" | "success";
  let passwordStep = $state<PasswordStep>("verify");

  let verificationSessionId = $state<string | null>(null);
  let verificationMethod = $state<"totp" | "email" | null>(null);
  let verificationCode = $state("");
  let verificationError = $state<string | null>(null);
  let verificationBusy = $state(false);
  let emailTotpSent = $state(false);

  let newPassword = $state("");
  let confirmPassword = $state("");
  let passwordError = $state<string | null>(null);
  let passwordBusy = $state(false);

  // Initialize verification method when data loads (only once)
  $effect(() => {
    if (verificationMethod === null && !pageData.loading && pageData.data) {
      // Set default based on whether TOTP is enabled
      verificationMethod = pageData.data.totpEnabled ? "totp" : "email";
    }
  });

  const switchToEmail = () => {
    verificationMethod = "email";
    verificationCode = "";
    verificationError = null;
  };

  const switchToTotp = () => {
    verificationMethod = "totp";
    verificationCode = "";
    verificationError = null;
  };

  const requestEmailTotp = async () => {
    const token = auth.getToken();
    if (!token) {
      verificationError = "Not authenticated";
      return;
    }

    verificationBusy = true;
    verificationError = null;

    try {
      await authCommands.requestEmailTotp("password_change", fetch, token);
      emailTotpSent = true;
    } catch (e) {
      verificationError = e instanceof Error ? e.message : "Failed to send verification code";
    } finally {
      verificationBusy = false;
    }
  };

  const verifyEmailTotp = async () => {
    const token = auth.getToken();
    if (!token) {
      verificationError = "Not authenticated";
      return;
    }

    const code = verificationCode.trim();
    if (!code || code.length !== 6) {
      verificationError = "Enter the 6-digit code from your email";
      return;
    }

    verificationBusy = true;
    verificationError = null;

    try {
      const response = await authCommands.verifyEmailTotp(code, "password_change", fetch, token);
      verificationSessionId = response.verificationSessionId;
      passwordStep = "password";
    } catch (e) {
      verificationError = e instanceof Error ? e.message : "Invalid code";
    } finally {
      verificationBusy = false;
    }
  };

  const verifyTotp = async () => {
    const token = auth.getToken();
    if (!token) {
      verificationError = "Not authenticated";
      return;
    }

    const code = verificationCode.trim();
    if (!code) {
      verificationError = "Enter the code from your authenticator app";
      return;
    }

    verificationBusy = true;
    verificationError = null;

    try {
      const response = await authCommands.verifyTotp(code, "password_change", fetch, token);
      verificationSessionId = response.verificationSessionId;
      passwordStep = "password";
    } catch (e) {
      verificationError = e instanceof Error ? e.message : "Invalid code";
    } finally {
      verificationBusy = false;
    }
  };

  const changePassword = async () => {
    const token = auth.getToken();
    if (!token) {
      passwordError = "Not authenticated";
      return;
    }

    if (!verificationSessionId) {
      passwordError = "Verification required";
      return;
    }

    if (!newPassword) {
      passwordError = "Enter a new password";
      return;
    }

    if (newPassword.length < 12) {
      passwordError = "Password must be at least 12 characters";
      return;
    }

    if (newPassword !== confirmPassword) {
      passwordError = "Passwords do not match";
      return;
    }

    passwordBusy = true;
    passwordError = null;

    try {
      await authCommands.changePasswordWithVerification(
        { verificationSessionId, newPassword },
        fetch,
        token
      );
      passwordStep = "success";
    } catch (e) {
      passwordError = e instanceof Error ? e.message : "Failed to change password";
    } finally {
      passwordBusy = false;
    }
  };

  const goToLogin = async () => {
    await auth.logout();
  };
</script>

{#if pageData.loading}
  <PageLoading presentation="inline" message="Loading..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if passwordStep === "verify"}
  <div class="intro">
    <p>Before changing your password, please verify your identity.</p>
  </div>

  {#if verificationError}
    <PoodleCallout tone="danger" message={verificationError} announceMode="polite" />
  {/if}

  {#if verificationMethod === null}
    <!-- Waiting for method to be determined -->
  {:else if verificationMethod === "totp"}
    <PoodleCard>
      <p class="muted">Enter the 6-digit code from your authenticator app.</p>
      <PoodleCodeInput
        value={verificationCode}
        label="Authenticator code"
        disabled={verificationBusy}
        on:valueChange={(event) => { verificationCode = event.detail.value; }}
        on:complete={verifyTotp}
      />
      <PoodleFormActions align="start">
        <PoodleButton type="button" variant="primary" disabled={verificationBusy} on:click={verifyTotp}>
          {verificationBusy ? "Verifying..." : "Verify"}
        </PoodleButton>
      </PoodleFormActions>
    </PoodleCard>
    <p class="switch-method">
      <PoodleButton variant="ghost" disabled={verificationBusy} on:click={switchToEmail}>
        Send code via email instead
      </PoodleButton>
    </p>
  {:else}
    {#if emailTotpSent}
      <PoodleCard>
        <p class="muted">We've sent a 6-digit code to your email address. Enter it below.</p>
        <PoodleCodeInput
          value={verificationCode}
          label="Email code"
          disabled={verificationBusy}
          on:valueChange={(event) => { verificationCode = event.detail.value; }}
          on:complete={verifyEmailTotp}
        />
        <PoodleFormActions align="start">
          <PoodleButton type="button" variant="primary" disabled={verificationBusy} on:click={verifyEmailTotp}>
            {verificationBusy ? "Verifying..." : "Verify"}
          </PoodleButton>
          <PoodleButton type="button" variant="secondary" disabled={verificationBusy} on:click={requestEmailTotp}>
            Resend Code
          </PoodleButton>
        </PoodleFormActions>
      </PoodleCard>
    {:else}
      <PoodleCard>
        <p class="muted">We'll send a verification code to your email address.</p>
        <PoodleFormActions align="start">
          <PoodleButton type="button" variant="primary" disabled={verificationBusy} on:click={requestEmailTotp}>
            {verificationBusy ? "Sending..." : "Send Verification Code"}
          </PoodleButton>
        </PoodleFormActions>
      </PoodleCard>
    {/if}
    {#if pageData.data?.totpEnabled}
      <p class="switch-method">
        <PoodleButton variant="ghost" disabled={verificationBusy} on:click={switchToTotp}>
          Use authenticator app instead
        </PoodleButton>
      </p>
    {/if}
  {/if}
{:else if passwordStep === "password"}
  <PoodleCard>
    {#if passwordError}
      <PoodleCallout tone="danger" message={passwordError} announceMode="polite" />
    {/if}

    <PasswordRequirements
      password={newPassword}
      fetchRequirements={() => authCommands.passwordRequirements(fetch)}
    />

    <div class="password-form">
      <PoodleField id="account-new-password" label="New password" let:describedBy>
        <PoodleTextInput
          id="account-new-password"
          value={newPassword}
          describedBy={describedBy}
          type="password"
          disabled={passwordBusy}
          on:valueChange={(event) => { newPassword = event.detail.value; }}
        />
      </PoodleField>

      <PoodleField id="account-confirm-password" label="Confirm new password" let:describedBy>
        <PoodleTextInput
          id="account-confirm-password"
          value={confirmPassword}
          describedBy={describedBy}
          type="password"
          disabled={passwordBusy}
          on:valueChange={(event) => { confirmPassword = event.detail.value; }}
        />
      </PoodleField>

      <PoodleFormActions align="start">
        <PoodleButton type="button" variant="primary" disabled={passwordBusy} on:click={changePassword}>
          {passwordBusy ? "Changing..." : "Change Password"}
        </PoodleButton>
      </PoodleFormActions>
    </div>
  </PoodleCard>
{:else if passwordStep === "success"}
  <div class="intro">
    <p class="success-message">Your password has been changed successfully.</p>
    <p class="muted">For security, you've been logged out of all sessions and will need to sign in again with your new password.</p>
  </div>
  <PoodleFormActions align="start">
    <PoodleButton type="button" variant="primary" on:click={goToLogin}>
      Sign In
    </PoodleButton>
  </PoodleFormActions>
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

  .success-message {
    color: #22c55e;
  }

  .switch-method {
    margin-top: 1rem;
  }

  .password-form {
    max-width: 24rem;
    margin-top: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
</style>
