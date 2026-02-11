<script lang="ts">
  import {
    Field,
    FieldSet,
    FieldSetGrid,
    FormActions,
    FormValidationProvider,
    SaveSplitButton,
    Select,
    Switch,
    TextButton,
    TextInput
  } from "@decodelabs/underlay/components";
  import { navigateOnCancel } from "@decodelabs/underlay/client";
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

  let isFormValid = $state(false);

  function handleCancel() {
    navigateOnCancel(cancelHref);
  }
</script>

<FormValidationProvider bind:isValid={isFormValid}>
  <FieldSet legend="User">
    <FieldSetGrid columns={2}>
	<Field label="Email" error={errors?.email} required>
		<TextInput
			name="email"
			value={emailValue}
			onchange={(v) => { emailValue = v; }}
			required
			placeholder="name@example.com"
			maxlength={320}
		/>
	</Field>

    <Field label="Display Name" error={errors?.displayName}>
      <TextInput
        name="displayName"
        value={displayNameValue}
        onchange={(v) => { displayNameValue = v; }}
        placeholder="Optional"
        maxlength={100}
      />
    </Field>
    </FieldSetGrid>
  </FieldSet>

  <FieldSet legend="Access">
    <FieldSetGrid columns={2}>
    <Field label="Role" error={errors?.role} required>
      <Select
        name="role"
        value={roleValue}
        onchange={(v) => { roleValue = v; }}
        items={roleItems}
        placeholder="Select role"
      />
    </Field>

    <Field label="Status" error={errors?.status} required>
      <Select
        name="status"
        value={statusValue}
        onchange={(v) => { statusValue = v; }}
        items={statusItems}
        placeholder="Select status"
      />
    </Field>
    </FieldSetGrid>
  </FieldSet>

  {#if mode === "create"}
    <FieldSet legend="Onboarding">
      <FieldSetGrid columns={1}>
      <Field
        label="Send password reset email"
        hint="Sends a password reset email so the user can set an initial password."
        error={errors?.sendPasswordReset}
      >
        <input
          type="hidden"
          name="sendPasswordReset"
          value={sendPasswordResetValue ? "true" : "false"}
        />
        <Switch
          name="sendPasswordReset-switch"
          bind:checked={sendPasswordResetValue}
          leftLabel="No"
          rightLabel="Yes"
          leftVariant="default"
          rightVariant="success"
        />
      </Field>
      </FieldSetGrid>
    </FieldSet>
  {/if}
</FormValidationProvider>

<FormActions align="start">
  {#snippet danger()}
    {#if returnTo}
      <input type="hidden" name="returnTo" value={returnTo} />
    {/if}

    <TextButton type="button" onclick={handleCancel}>
      Cancel
    </TextButton>
  {/snippet}

  <SaveSplitButton
    mode={mode}
    disabled={!isFormValid}
    createLabel="Create user"
    saveLabel="Save changes"
    saveAndCloseLabel="Save & close"
    bind:intent
  />
</FormActions>
