<script lang="ts">
  import {
    Field,
    FieldSet,
    FieldSetGrid,
    FormActions,
    FormValidationProvider,
    SaveSplitButton,
    Switch,
    TextButton,
    TextInput,
    TextArea
  } from "@decodelabs/underlay/components";
  import { SlugField } from "@decodelabs/underlay/patterns";
  import type { ValidationResult } from "@decodelabs/underlay/components";
  import { navigateOnCancel } from "@decodelabs/underlay/client";
  import { untrack } from "svelte";

  type CategoryFormMode = "create" | "edit";

  interface CategoryFormValues {
    name?: string;
    slug?: string;
    description?: string | null;
    color?: string | null;
    isActive?: boolean;
  }

  interface Props {
    mode?: CategoryFormMode;
    categoryId?: string;
    values?: CategoryFormValues;
    /** Async function to validate slug availability */
    validateSlug?: (slug: string) => Promise<ValidationResult>;
    intent?: "save" | "save-close";
    errors?: Record<string, string> | null;
    cancelHref?: string;
    returnTo?: string;
    prepare?: (formData: FormData) => void;
  }

  let {
    mode = "edit",
    categoryId = undefined,
    values = {},
    validateSlug,
    intent = $bindable("save-close"),
    errors = null,
    cancelHref = undefined,
    returnTo = undefined,
    prepare = $bindable(() => {})
  }: Props = $props();

  // Local state for form fields
  let nameValue = $state(untrack(() => values.name ?? ""));
  let slugValue = $state(untrack(() => values.slug ?? ""));
  let descriptionValue = $state(untrack(() => values.description ?? ""));
  let colorValue = $state(untrack(() => values.color ?? "#6366f1"));
  let isActive = $state(untrack(() => values.isActive ?? true));

  // Form validation state
  let isFormValid = $state(false);

  function handleCancel() {
    navigateOnCancel(cancelHref);
  }
</script>

<FormValidationProvider bind:isValid={isFormValid}>
  <FieldSet legend="Basic Information">
    <FieldSetGrid columns={2}>
    <Field label="Name" error={errors?.name} required>
      <TextInput
        name="name"
        bind:value={nameValue}
        required
        placeholder="e.g., Development"
        maxlength={64}
      />
    </Field>

    <SlugField
      name="slug"
      label="Slug"
      bind:value={slugValue}
      source={nameValue}
      validate={validateSlug}
      maxlength={64}
      error={errors?.slug}
    />

    <Field label="Description" error={errors?.description} span="full">
      <TextArea
        name="description"
        bind:value={descriptionValue}
        placeholder="Optional description for this category"
        rows={3}
        maxlength={500}
      />
    </Field>
    </FieldSetGrid>
  </FieldSet>

  <FieldSet legend="Display Settings">
    <FieldSetGrid columns={2}>
    <Field label="Color" error={errors?.color}>
      <input
        type="color"
        name="color"
        bind:value={colorValue}
        class="color-picker"
      />
    </Field>

    <Field label="Status" error={errors?.isActive}>
      <Switch
        name="isActive"
        bind:checked={isActive}
        leftLabel="Inactive"
        rightLabel="Active"
        leftVariant="danger"
        rightVariant="success"
      />
    </Field>
    </FieldSetGrid>
  </FieldSet>
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
    disabled={!isFormValid}
    bind:intent
  />
</FormActions>

<style>
  .color-picker {
    width: 4rem;
    height: 2.5rem;
    padding: 0.25rem;
    border: 1px solid var(--underlay-color-border-subtle, #e5e7eb);
    border-radius: 0.375rem;
    cursor: pointer;
  }
</style>
