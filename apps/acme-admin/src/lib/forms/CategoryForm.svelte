<script lang="ts">
import {
  isReservedSlug,
  isValidSlugFormat,
  slugify
} from "@inflatable-cookie/underlay/utils/slug";
import {
  Button,
  ColorPicker,
  Field,
  FieldSet,
  FormActions,
  SplitButton,
  Switch,
  TextInput,
  type InputValidationStatus,
  } from "@inflatable-cookie/poodle-svelte";
  import type { ValidationResult } from "@inflatable-cookie/poodle-svelte";
    import { navigateOnCancel } from "@inflatable-cookie/underlay/client/navigation";
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
  let lastAutoSlug = $state(untrack(() => slugify(values.name ?? "")));
  let slugStatus = $state<InputValidationStatus>("idle");
  let slugValidationMessage = $state<string | null>(null);

  const editIntentItems = [
    { value: "save", label: "Save changes" },
    { value: "save-close", label: "Save & close" }
  ];

  const createIntentItems = [
    { value: "save", label: "Create & continue" },
    { value: "save-close", label: "Create & close" }
  ];

  let actionBarElement = $state<HTMLDivElement | null>(null);

  const isFormValid = $derived.by(() => {
    return Boolean(
      nameValue.trim() &&
      slugValue.trim() &&
      slugStatus !== "invalid" &&
      slugStatus !== "validating"
    );
  });

  const effectiveSlugError = $derived(errors?.slug ?? slugValidationMessage);
  const slugFieldValidationState = $derived.by(() => {
    if (effectiveSlugError) {
      return "invalid";
    }
    if (slugStatus === "validating") {
      return "pending";
    }
    if (slugStatus === "valid") {
      return "valid";
    }
    return "none";
  });

  $effect(() => {
    const nextAutoSlug = slugify(nameValue);
    if (!slugValue.trim() || slugValue === lastAutoSlug) {
      slugValue = nextAutoSlug;
    }
    lastAutoSlug = nextAutoSlug;
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

  async function validateCategorySlug(slug: string): Promise<ValidationResult> {
    const normalizedSlug = slug.trim();

    if (normalizedSlug.length < 2) {
      return { valid: false, message: "Too short (min 2 characters)" };
    }

    if (normalizedSlug.length > 64) {
      return { valid: false, message: "Too long (max 64 characters)" };
    }

    if (!isValidSlugFormat(normalizedSlug, 64)) {
      return {
        valid: false,
        message: "Invalid format (use lowercase letters, numbers, hyphens)",
      };
    }

    if (isReservedSlug(normalizedSlug)) {
      return { valid: false, message: "This slug is reserved" };
    }

    if (!validateSlug) {
      return { valid: true };
    }

    return validateSlug(normalizedSlug);
  }

  function handleSlugBlur() {
    if (!slugValue) {
      return;
    }

    const normalizedSlug = slugify(slugValue);
    if (normalizedSlug !== slugValue) {
      slugValue = normalizedSlug;
    }
  }
</script>

<FieldSet legend="Basic Information" columns={2}>
    <Field
      id="category-name"
      label="Name"
      error={errors?.name ?? null}
      validationState={validationState(errors?.name)}
      required
    >
      {#snippet control({ describedBy, validationState })}
        <TextInput
          id="category-name"
          name="name"
          value={nameValue}
          describedBy={describedBy}
          validationState={validationState}
          placeholder="e.g., Development"
          maxLength={64}
          onValueChange={(nextValue) => { nameValue = nextValue; }}
        />
      {/snippet}
    </Field>

    <Field
      id="category-slug"
      label="Slug"
      error={effectiveSlugError}
      validationState={slugFieldValidationState}
      description="Used in URLs, lowercase letters and hyphens only."
      required
    >
      {#snippet control({ describedBy, validationState })}
        <TextInput
          id="category-slug"
          name="slug"
          value={slugValue}
          describedBy={describedBy}
          validationState={validationState}
          placeholder="e.g., development"
          autocomplete="off"
          required
          pattern="[a-z0-9]+(?:-[a-z0-9]+)*"
          maxLength={64}
          validate={validateCategorySlug}
          validationContext={{ categoryId }}
          validationDebounce={300}
          onValueChange={(nextValue) => { slugValue = nextValue; }}
          onValidationChange={(detail) => {
            slugStatus = detail.status;
            slugValidationMessage = detail.status === "invalid" ? detail.message || null : null;
          }}
          onBlur={handleSlugBlur}
        />
      {/snippet}
    </Field>

    <Field
      id="category-description"
      label="Description"
      error={errors?.description ?? null}
      validationState={validationState(errors?.description)}
      span="full"
    >
      {#snippet control({ describedBy, validationState })}
        <TextInput
          id="category-description"
          name="description"
          value={descriptionValue}
          describedBy={describedBy}
          validationState={validationState}
          placeholder="Optional description for this category"
          rows={3}
          onValueChange={(nextValue) => { descriptionValue = nextValue; }}
        />
      {/snippet}
    </Field>
</FieldSet>

<FieldSet legend="Display Settings" columns={2}>
    <Field
      id="category-color"
      label="Color"
      error={errors?.color ?? null}
      validationState={validationState(errors?.color)}
    >
      {#snippet control()}
        <input type="hidden" name="color" value={colorValue} />
        <ColorPicker
          value={colorValue}
          ariaLabel="Category colour"
          onChange={(nextValue) => { colorValue = nextValue; }}
        />
      {/snippet}
    </Field>

    <Field
      id="category-status"
      label="Status"
      error={errors?.isActive ?? null}
      validationState={validationState(errors?.isActive)}
    >
      {#snippet control({ describedBy })}
        <input type="hidden" name="isActive" value={isActive ? "true" : "false"} />
        <Switch
          id="category-status"
          checked={isActive}
          describedBy={describedBy}
          ariaLabel="Category status"
          label={isActive ? "Active" : "Inactive"}
          onCheckedChange={(checked) => { isActive = checked; }}
        />
      {/snippet}
    </Field>
</FieldSet>

<FormActions align="end" showTopBorder>
  <div class="category-form__actions" bind:this={actionBarElement}>
    <input type="hidden" name="intent" value={intent} />

    {#if returnTo}
      <input type="hidden" name="returnTo" value={returnTo} />
    {/if}

    <Button type="button" variant="ghost" onClick={handleCancel}>
      Cancel
    </Button>

    <SplitButton
      variant="primary"
      items={mode === "create" ? createIntentItems : editIntentItems}
      disabled={!isFormValid}
      onClick={() => submitWithIntent(intent)}
      onAction={(value) => submitWithIntent(value as "save" | "save-close")}
    >
      {#if mode === "create"}
        {intent === "save" ? "Create & continue" : "Create & close"}
      {:else}
        {intent === "save" ? "Save changes" : "Save & close"}
      {/if}
    </SplitButton>
  </div>
</FormActions>

<style>
  .category-form__actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--poodle-space-inline-md);
  }
</style>
