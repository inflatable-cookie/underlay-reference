<script lang="ts">
  import {
    Button,
    Card,
    Field,
    FormActions,
    FormError,
    TextInput,
    TextButton,
    TotpInput,
    PasswordRequirements,
    PageLoading
  } from "@decodelabs/underlay/components";
  import { authCommands } from "acme-client";
  import { auth } from "$lib/stores/auth";
  import { useAuthenticatedData } from "@decodelabs/underlay/patterns";

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
  <PageLoading message="Loading..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if passwordStep === "verify"}
  <div class="intro">
    <p>Before changing your password, please verify your identity.</p>
  </div>

  {#if verificationError}
    <FormError message={verificationError} />
  {/if}

  {#if verificationMethod === null}
    <!-- Waiting for method to be determined -->
  {:else if verificationMethod === "totp"}
    <Card>
      <p class="muted">Enter the 6-digit code from your authenticator app.</p>
      <TotpInput
        bind:value={verificationCode}
        label="Authenticator code"
        disabled={verificationBusy}
        oncomplete={verifyTotp}
      />
      <FormActions>
        <Button type="button" variant="primary" onclick={verifyTotp} disabled={verificationBusy}>
          {verificationBusy ? "Verifying..." : "Verify"}
        </Button>
      </FormActions>
    </Card>
    <p class="switch-method">
      <TextButton onclick={switchToEmail} disabled={verificationBusy}>
        Send code via email instead
      </TextButton>
    </p>
  {:else}
    {#if emailTotpSent}
      <Card>
        <p class="muted">We've sent a 6-digit code to your email address. Enter it below.</p>
        <TotpInput
          bind:value={verificationCode}
          label="Email code"
          disabled={verificationBusy}
          oncomplete={verifyEmailTotp}
        />
        <FormActions>
          <Button type="button" variant="primary" onclick={verifyEmailTotp} disabled={verificationBusy}>
            {verificationBusy ? "Verifying..." : "Verify"}
          </Button>
          <Button type="button" variant="secondary" onclick={requestEmailTotp} disabled={verificationBusy}>
            Resend Code
          </Button>
        </FormActions>
      </Card>
    {:else}
      <Card>
        <p class="muted">We'll send a verification code to your email address.</p>
        <FormActions>
          <Button type="button" variant="primary" onclick={requestEmailTotp} disabled={verificationBusy}>
            {verificationBusy ? "Sending..." : "Send Verification Code"}
          </Button>
        </FormActions>
      </Card>
    {/if}
    {#if pageData.data?.totpEnabled}
      <p class="switch-method">
        <TextButton onclick={switchToTotp} disabled={verificationBusy}>
          Use authenticator app instead
        </TextButton>
      </p>
    {/if}
  {/if}
{:else if passwordStep === "password"}
  <Card>
    {#if passwordError}
      <FormError message={passwordError} />
    {/if}

    <PasswordRequirements
      password={newPassword}
      fetchRequirements={() => authCommands.passwordRequirements(fetch)}
    />

    <div class="password-form">
      <Field label="New password">
        <TextInput
          bind:value={newPassword}
          type="password"
          autocomplete="new-password"
          disabled={passwordBusy}
        />
      </Field>

      <Field label="Confirm new password">
        <TextInput
          bind:value={confirmPassword}
          type="password"
          autocomplete="new-password"
          disabled={passwordBusy}
        />
      </Field>

      <FormActions>
        <Button type="button" variant="primary" onclick={changePassword} disabled={passwordBusy}>
          {passwordBusy ? "Changing..." : "Change Password"}
        </Button>
      </FormActions>
    </div>
  </Card>
{:else if passwordStep === "success"}
  <div class="intro">
    <p class="success-message">Your password has been changed successfully.</p>
    <p class="muted">For security, you've been logged out of all sessions and will need to sign in again with your new password.</p>
  </div>
  <FormActions>
    <Button type="button" variant="primary" onclick={goToLogin}>
      Sign In
    </Button>
  </FormActions>
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
