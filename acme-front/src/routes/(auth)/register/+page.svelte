<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/stores/auth";
  import { Button, Callout, Field, TextInput } from "@poodle/svelte";

  let displayName = $state("");
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
      await auth.register(email.trim(), password, displayName.trim());
      await goto("/dashboard");
    } catch (e) {
      error = e instanceof Error ? e.message : "Registration failed";
    } finally {
      submitting = false;
    }
  }
</script>

<h1>Create Account</h1>

<form onsubmit={handleSubmit}>
  {#if error}
    <Callout tone="danger" message={error} announceMode="assertive" />
  {/if}

  <Field id="register-name" label="Name" required>
    {#snippet control({ describedBy })}
      <TextInput
        id="register-name"
        value={displayName}
        describedBy={describedBy}
        placeholder="Your name"
        disabled={submitting}
        onValueChange={(nextValue) => { displayName = nextValue; }}
      />
    {/snippet}
  </Field>

  <Field id="register-email" label="Email" required>
    {#snippet control({ describedBy })}
      <TextInput
        id="register-email"
        type="email"
        value={email}
        describedBy={describedBy}
        placeholder="you@example.com"
        disabled={submitting}
        onValueChange={(nextValue) => { email = nextValue; }}
      />
    {/snippet}
  </Field>

  <Field id="register-password" label="Password" required>
    {#snippet control({ describedBy })}
      <TextInput
        id="register-password"
        type="password"
        value={password}
        describedBy={describedBy}
        placeholder="Choose a password"
        disabled={submitting}
        onValueChange={(nextValue) => { password = nextValue; }}
      />
    {/snippet}
  </Field>

  <div class="form-actions">
    <Button type="submit" variant="primary" disabled={submitting}>
      {submitting ? "Creating account..." : "Create Account"}
    </Button>
  </div>

  <p class="links">
    Already have an account? <a href="/login">Sign in</a>
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
