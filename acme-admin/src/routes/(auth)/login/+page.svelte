<script lang="ts">
import {
  LoginPage
} from "@decodelabs/underlay/patterns";
import {
  browser } from "$app/environment";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { auth,
  authLoading,
  currentUser } from "$lib/stores/auth";
  import { authCommands } from "@api-client";
    import { toPublicKeyRequestOptions, assertionToJson } from "@decodelabs/underlay/utils/webauthn";

  // Initialize auth and redirect if already logged in
  onMount(async () => {
    await auth.initialize();
  });

  // Track if we're showing setup prompt (to prevent redirect)
  let inSetupPrompt = $state(false);
  // Redirect to dashboard if authenticated (but not if showing setup prompt)
  $effect(() => {
    if (!$authLoading && $currentUser && !inSetupPrompt) {
      goto('/');
    }
  });

  // Password login handler - returns LoginResult for the component
  async function handlePasswordLogin(email: string, password: string) {
    const result = await auth.loginStart(email, password);

    if (result.requiresTwoFactor) {
      if (result.isEmailVerification) {
        inSetupPrompt = true;
      }
      return {
        requiresTwoFactor: true,
        isEmailVerification: result.isEmailVerification,
        loginStateId: result.loginStateId,
        email
      };
    }

    return { complete: true };
  }

  // Passkey login handler
  async function handlePasskeyLogin(email?: string) {
    if (!browser || typeof window === "undefined" || !("PublicKeyCredential" in window) || !navigator.credentials) {
      throw new Error("Passkeys are not supported in this browser.");
    }

    // Start passkey login
    const startData = await authCommands.passkeyLoginStart({ email }, fetch);
    const publicKey = toPublicKeyRequestOptions(startData.options as any);

    if (!(publicKey as any).challenge) {
      throw new Error("Passkey login failed: server did not return a challenge.");
    }

    const cred = (await navigator.credentials.get({
      publicKey
    })) as PublicKeyCredential | null;

    if (!cred) {
      throw new Error("Passkey login was cancelled.");
    }

    // Complete passkey login
    const loginResponse = await authCommands.passkeyLoginFinish({
      stateId: startData.stateId,
      credential: assertionToJson(cred) as any
    }, fetch);

    // Store session
    auth.setSession(loginResponse);
  }

  // 2FA verification handler
  async function handleTwoFactorVerify(stateId: string, code: string) {
    await auth.loginFinish(stateId, code);
  }

  // Login complete handler
  function handleComplete() {
    goto('/');
  }

  function handleSkipSetup() {
    inSetupPrompt = false;
  }
</script>

<h1 class="admin-login__title">Acme</h1>

<LoginPage
  methods={['password', 'passkey']}
  onPasswordLogin={handlePasswordLogin}
  onTwoFactorVerify={handleTwoFactorVerify}
  onPasskeyLogin={handlePasskeyLogin}
  showPasskeyEmailField={true}
  onComplete={handleComplete}
  forgotPasswordHref="/forgot-password"
  showSetupPrompt={true}
  setupHref="/account/2fa"
  onSkipSetup={handleSkipSetup}
  passkeyHint="Passkeys let you sign in using your device, a password manager, or a security key. If you have multiple accounts, you can optionally enter your email to narrow the choice."
/>

<style>
  .admin-login__title {
    text-align: center;
    font-size: 1.5rem;
    font-weight: 650;
    margin: 0 0 1.25rem;
    letter-spacing: 0.04em;
  }
</style>
