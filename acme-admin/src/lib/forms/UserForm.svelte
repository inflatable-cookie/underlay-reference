<script lang="ts">
  import {
    Button,
    Field,
    FieldSet,
    FormActions,
    SplitButton,
    Select,
    Switch,
    TextInput
  } from "@poodle/svelte";
  import { navigateOnCancel } from "@decodelabs/underlay/client/navigation";
  import { untrack } from "svelte";

  type UserFormMode = "create" | "edit";

  interface UserFormValues {
    email?: string;
    displayName?: string | null;
    role?: string;
    status?: string;
    sendPasswordReset?: boolean;
  }

  interface Props {
    mode?: UserFormMode;
    values?: UserFormValues;
    intent?: "save" | "save-close";
    errors?: Record<string, string> | null;
    cancelHref?: string;
    returnTo?: string;
  }

  let {
    mode = "edit",
    values = {},
    intent = $bindable("save-close"),
    errors = null,
    cancelHref = undefined,
    returnTo = undefined
  }: Props = $props();

  let emailValue = $state(untrack(() => values.email ?? ""));
  let displayNameValue = $state(untrack(() => values.displayName ?? ""));
  let roleValue = $state(untrack(() => values.role ?? "user"));
  let statusValue = $state(untrack(() => values.status ?? "active"));
  let sendPasswordResetValue = $state(untrack(() => values.sendPasswordReset ?? true));

  const roleItems = [
    { value: "user", label: "User" },
    { value: "tester", label: "Tester" },
    { value: "editor", label: "Editor" },
    { value: "admin", label: "Admin" },
    { value: "support", label: "Support" },
    { value: "superadmin", label: "Superadmin" }
  ];

  const statusItems = [
    { value: "active", label: "Active" },
    { value: "suspended", label: "Suspended" },
    { value: "deleted", label: "Deleted" }
  ];

  const editIntentItems = [
    { value: "save", label: "Save changes" },
    { value: "save-close", label: "Save & close" }
  ];

  let actionBarElement = $state<HTMLDivElement | null>(null);

  const isFormValid = $derived.by(() => {
    const email = emailValue.trim();
    return Boolean(email && email.includes("@") && roleValue && statusValue);
  });

  $effect(() => {
    if (mode === "create") {
      intent = "save-close";
    }
  });

  function handleCancel() {
    navigateOnCancel(cancelHref);
  }

  function validationState(error?: string | null) {
    return error ? "invalid" : "none";
  }

  function submitWithIntent(nextIntent: "save" | "save-close") {
    intent = nextIntent;
    actionBarElement?.closest("form")?.requestSubmit();
  }
</script>

<FieldSet legend="User" columns={2}>
  <Field
    id="user-email"
    label="Email"
    error={errors?.email ?? null}
    validationState={validationState(errors?.email)}
    required
  >
    {#snippet control({ describedBy, validationState })}
      <TextInput
        id="user-email"
        name="email"
        type="email"
        inputMode="email"
        value={emailValue}
        describedBy={describedBy}
        validationState={validationState}
        placeholder="name@example.com"
        maxLength={320}
        onValueChange={(nextValue) => { emailValue = nextValue; }}
      />
    {/snippet}
  </Field>

  <Field
    id="user-display-name"
    label="Display Name"
    error={errors?.displayName ?? null}
    validationState={validationState(errors?.displayName)}
  >
    {#snippet control({ describedBy, validationState })}
      <TextInput
        id="user-display-name"
        name="displayName"
        value={displayNameValue}
        describedBy={describedBy}
        validationState={validationState}
        placeholder="Optional"
        maxLength={100}
        onValueChange={(nextValue) => { displayNameValue = nextValue; }}
      />
    {/snippet}
  </Field>
</FieldSet>

<FieldSet legend="Access" columns={2}>
  <Field
    id="user-role"
    label="Role"
    error={errors?.role ?? null}
    validationState={validationState(errors?.role)}
    required
  >
    {#snippet control({ describedBy })}
      <Select
        id="user-role"
        name="role"
        value={roleValue}
        describedBy={describedBy}
        options={roleItems}
        placeholder="Select role"
        onValueChange={(value) => { roleValue = value; }}
      />
    {/snippet}
  </Field>

  <Field
    id="user-status"
    label="Status"
    error={errors?.status ?? null}
    validationState={validationState(errors?.status)}
    required
  >
    {#snippet control({ describedBy })}
      <Select
        id="user-status"
        name="status"
        value={statusValue}
        describedBy={describedBy}
        options={statusItems}
        placeholder="Select status"
        onValueChange={(value) => { statusValue = value; }}
      />
    {/snippet}
  </Field>
</FieldSet>

{#if mode === "create"}
  <FieldSet legend="Onboarding">
    <Field
      id="user-send-password-reset"
      label="Send password reset email"
      hint="Sends a password reset email so the user can set an initial password."
      error={errors?.sendPasswordReset ?? null}
      validationState={validationState(errors?.sendPasswordReset)}
    >
      {#snippet control({ describedBy })}
        <input
          type="hidden"
          name="sendPasswordReset"
          value={sendPasswordResetValue ? "true" : "false"}
        />
        <div class="user-form__switch-row">
          <span class="user-form__switch-label">No</span>
          <Switch
            id="user-send-password-reset"
            name="sendPasswordReset-switch"
            checked={sendPasswordResetValue}
            describedBy={describedBy}
            ariaLabel="Send password reset email"
            onCheckedChange={(checked) => { sendPasswordResetValue = checked; }}
          />
          <span class="user-form__switch-label">Yes</span>
        </div>
      {/snippet}
    </Field>
  </FieldSet>
{/if}

<FormActions align="end">
  <div class="user-form__actions" bind:this={actionBarElement}>
    <input type="hidden" name="intent" value={intent} />

    {#if returnTo}
      <input type="hidden" name="returnTo" value={returnTo} />
    {/if}

    <Button type="button" variant="ghost" onClick={handleCancel}>
      Cancel
    </Button>

    {#if mode === "create"}
      <Button type="submit" variant="primary" disabled={!isFormValid}>
        Create user
      </Button>
    {:else}
      <SplitButton
        variant="primary"
        items={editIntentItems}
        disabled={!isFormValid}
        onClick={() => submitWithIntent(intent)}
        onAction={(value) => submitWithIntent(value as "save" | "save-close")}
      >
        {intent === "save" ? "Save changes" : "Save & close"}
      </SplitButton>
    {/if}
  </div>
</FormActions>

<style>
  .user-form__actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--poodle-space-inline-md);
  }

  .user-form__switch-row {
    display: inline-flex;
    align-items: center;
    gap: var(--poodle-space-inline-sm);
  }

  .user-form__switch-label {
    color: var(--poodle-color-text-secondary);
    font-family: var(--poodle-typography-label-family);
    font-size: 0.75rem;
    line-height: var(--poodle-typography-label-lineHeight);
  }
</style>
