<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/stores/auth";
  import { Button, Input, FormField, FormError } from "@decodelabs/underlay/components";

  let email = $state("");
  let password = $state("");
  let submitting = $state(false);
  let error = $state<string | null>(null);

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (submitting) return;

    submitting = true;
    error = null;

    try {
      await auth.login(email.trim(), password);
      await goto("/dashboard");
    } catch (e) {
      error = e instanceof Error ? e.message : "Login failed";
    } finally {
      submitting = false;
    }
  }
</script>

<h1>Sign In</h1>

<form onsubmit={handleSubmit}>
  {#if error}
    <FormError message={error} />
  {/if}

  <FormField label="Email" required>
    <Input
      type="email"
      bind:value={email}
      placeholder="you@example.com"
      disabled={submitting}
      autocomplete="email"
    />
  </FormField>

  <FormField label="Password" required>
    <Input
      type="password"
      bind:value={password}
      placeholder="Your password"
      disabled={submitting}
      autocomplete="current-password"
    />
  </FormField>

  <div class="form-actions">
    <Button type="submit" variant="primary" disabled={submitting}>
      {submitting ? "Signing in..." : "Sign In"}
    </Button>
  </div>

  <p class="links">
    Don't have an account? <a href="/register">Sign up</a>
  </p>
</form>

<style>
  h1 {
    margin: 0 0 1.5rem;
    font-size: 1.5rem;
    text-align: center;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .form-actions {
    margin-top: 0.5rem;
  }

  .form-actions :global(button) {
    width: 100%;
  }

  .links {
    margin: 1rem 0 0;
    font-size: 0.875rem;
    text-align: center;
    color: var(--text-secondary, #6b7280);
  }
</style>
