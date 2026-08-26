<script lang="ts">
  import {
    Button,
    ColorPicker,
    Field,
    FieldSet,
    FormActions,
    TextInput
  } from "@inflatable-cookie/poodle-svelte";

  type LabelFormMode = "create" | "edit";

  interface Props {
    mode?: LabelFormMode;
    name?: string;
    color?: string;
    errors?: Record<string, string> | null;
    submitting?: boolean;
    cancelHref?: string;
    onCancel?: () => void;
  }

  const noop = () => {};

  let {
    mode = "edit",
    name = $bindable(""),
    color = $bindable("#6366f1"),
    errors = null,
    submitting = false,
    cancelHref = undefined,
    onCancel = noop
  }: Props = $props();

  function validationState(error?: string | null) {
    return error ? "invalid" : "none";
  }

  const isFormValid = $derived(Boolean(name.trim()));
</script>

<FieldSet legend="Label" columns={2}>
  <Field
    id={`label-${mode}-name`}
    label="Name"
    error={errors?.name ?? null}
    validationState={validationState(errors?.name)}
    required
  >
    {#snippet control({ describedBy, validationState })}
      <TextInput
        id={`label-${mode}-name`}
        name="name"
        value={name}
        describedBy={describedBy}
        validationState={validationState}
        placeholder="e.g., urgent"
        maxLength={64}
        disabled={submitting}
        onValueChange={(nextValue) => { name = nextValue; }}
      />
    {/snippet}
  </Field>

  <Field
    id={`label-${mode}-color`}
    label="Color"
    error={errors?.color ?? null}
    validationState={validationState(errors?.color)}
  >
    {#snippet control()}
      <input type="hidden" name="color" value={color} />
      <ColorPicker
        value={color}
        ariaLabel="Label colour"
        onChange={(nextValue) => { color = nextValue; }}
      />
    {/snippet}
  </Field>
</FieldSet>

<FormActions align="end" showTopBorder>
  <div class="label-form__actions">
    <Button type="button" variant="ghost" disabled={submitting} onClick={onCancel}>
      Cancel
    </Button>
    <Button type="submit" variant="primary" disabled={submitting || !isFormValid}>
      {#if mode === "create"}
        {submitting ? "Creating..." : "Create label"}
      {:else}
        {submitting ? "Saving..." : "Save Changes"}
      {/if}
    </Button>
  </div>
</FormActions>

<style>
  .label-form__actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--poodle-space-inline-md);
  }
</style>
