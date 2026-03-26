<script lang="ts">
  import {
    Button,
    Field,
    FieldSet,
    FormActions,
    SplitButton,
    Select,
    TextInput,
    TextArea
  } from "@poodle/svelte-primitives";
  import {
    RelationSelector,
    createLocalSearchFns,
    type SelectableRelation,
    type SearchResult,
    type SuggestionOptions
  } from "@decodelabs/underlay/patterns";
  import { navigateOnCancel } from "@decodelabs/underlay/client";
  import { categorySelectionHistory } from "$lib/stores/selection-history";
  import { untrack } from "svelte";
  import type { Category, CategoryWithCounts } from "acme-client";
  import CategoryForm from "./CategoryForm.svelte";

  type ProjectFormMode = "create" | "edit";

  interface ProjectFormValues {
    name?: string;
    description?: string | null;
    categoryId?: string | null;
    status?: string;
  }

  /** Converts a Category to SelectableRelation for RelationSelector */
  function categoryToSelectable(category: Category | CategoryWithCounts): SelectableRelation {
    return {
      id: category.id,
      label: category.name,
      description: category.description ?? undefined
    };
  }

  interface Props {
    mode?: ProjectFormMode;
    projectId?: string;
    values?: ProjectFormValues;
    /** Available categories for the selector */
    categories?: (Category | CategoryWithCounts)[];
    /** Function to fetch categories with suggestion options */
    fetchCategories?: (options?: { suggestions?: boolean; recentHints?: string[] }) => Promise<Category[]>;
    /** Function to create a new category inline */
    createCategory?: (name: string, slug: string, description: string | null, color: string | null) => Promise<Category>;
    intent?: "save" | "save-close";
    errors?: Record<string, string> | null;
    cancelHref?: string;
    returnTo?: string;
    prepare?: (formData: FormData) => void;
  }

  let {
    mode = "edit",
    projectId = undefined,
    values = {},
    categories = [],
    fetchCategories = undefined,
    createCategory = undefined,
    intent = $bindable("save-close"),
    errors = null,
    cancelHref = undefined,
    returnTo = undefined,
    prepare = $bindable(() => {})
  }: Props = $props();

  // Local search functions for Category RelationSelector
  const { search: searchCategories, suggest: suggestCategories } = createLocalSearchFns(
    () => categories,
    {
      toSelectable: categoryToSelectable,
      getSearchText: (c) => [c.name, c.slug, c.description ?? ""]
    }
  );

  // Local state for form fields
  let nameValue = $state(untrack(() => values.name ?? ""));
  let descriptionValue = $state(untrack(() => values.description ?? ""));
  let categoryId = $state<string | null>(untrack(() => values.categoryId ?? null));
  let statusValue = $state(untrack(() => values.status ?? "active"));

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
    return Boolean(nameValue.trim() && statusValue.trim());
  });

  // State for inline category creation
  let isCreatingCategory = $state(false);
  let createCategoryError = $state<string | null>(null);

  function createCategoryInlineSubmitHandler(onSuccess: (item: SelectableRelation) => void) {
    return async (formData: FormData) => {
      if (!createCategory) return;

      const name = String(formData.get("name") ?? "").trim();
      const slug = String(formData.get("slug") ?? "").trim();
      const description = String(formData.get("description") ?? "").trim() || null;
      const color = String(formData.get("color") ?? "#6366f1").trim();

      if (!name || !slug) {
        createCategoryError = "Name and slug are required";
        return;
      }

      isCreatingCategory = true;
      createCategoryError = null;

      try {
        const newCategory = await createCategory(name, slug, description, color);
        onSuccess(categoryToSelectable(newCategory));
      } catch (e) {
        createCategoryError = e instanceof Error ? e.message : "Failed to create category";
      } finally {
        isCreatingCategory = false;
      }
    };
  }

  function createCategoryInlineCancelHandler(onCancel: () => void) {
    return () => {
      createCategoryError = null;
      onCancel();
    };
  }

  // Search function for RelationSelector when using server-side search
  const searchCategoriesServer = async (query: string): Promise<SearchResult<SelectableRelation>> => {
    if (!fetchCategories) {
      return searchCategories(query);
    }
    const cats = await fetchCategories();
    const q = query.toLowerCase();
    const filtered = cats
      .filter(c => c.name.toLowerCase().includes(q) || c.slug.toLowerCase().includes(q))
      .map(categoryToSelectable);
    return { items: filtered, total: filtered.length };
  };

  // Suggestions function
  const suggestCategoriesServer = async (options?: SuggestionOptions): Promise<SelectableRelation[]> => {
    if (!fetchCategories) {
      return suggestCategories(options);
    }
    const cats = await fetchCategories({
      suggestions: true,
      recentHints: options?.recentHints
    });
    return cats.map(categoryToSelectable);
  };

  const statusItems = [
    { value: "active", label: "Active" },
    { value: "archived", label: "Archived" },
    { value: "on_hold", label: "On Hold" }
  ];

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

<FieldSet legend="Organisation" columns={2}>
    <Field
      id="project-category"
      label="Category"
      error={errors?.categoryId ?? null}
      validationState={validationState(errors?.categoryId)}
      hint="Optional: Organize projects into categories"
    >
      <input type="hidden" name="categoryId" value={categoryId ?? ""} />
      <RelationSelector
        label="Select Category"
        value={categoryId}
        onchange={(val) => { categoryId = val; }}
        search={fetchCategories ? searchCategoriesServer : searchCategories}
        suggestions={fetchCategories ? suggestCategoriesServer : suggestCategories}
        selectionHistory={categorySelectionHistory}
        placeholder="Select a category…"
        allowCreate={!!createCategory}
        createLabel="Add new category"
      >
        {#snippet createForm(onSuccess, onCancel)}
          <form
            class="inline-form"
            onsubmit={(e) => {
              e.preventDefault();
              const formData = new FormData(e.currentTarget);
              createCategoryInlineSubmitHandler(onSuccess)(formData);
            }}
          >
            <CategoryForm
              mode="create"
              errors={createCategoryError ? { name: createCategoryError } : null}
            />
            <div class="inline-form-actions">
              <Button type="submit" variant="primary" disabled={isCreatingCategory}>
                {isCreatingCategory ? "Creating..." : "Create"}
              </Button>
              <Button type="button" variant="secondary" on:click={createCategoryInlineCancelHandler(onCancel)}>
                Cancel
              </Button>
            </div>
          </form>
        {/snippet}
      </RelationSelector>
    </Field>

    <Field
      id="project-status"
      label="Status"
      error={errors?.status ?? null}
      validationState={validationState(errors?.status)}
      required
      let:describedBy
    >
      <Select
        id="project-status"
        name="status"
        value={statusValue}
        describedBy={describedBy}
        options={statusItems}
        placeholder="Select status"
        on:valueChange={(event) => { statusValue = event.detail.value; }}
      />
    </Field>
</FieldSet>

<FieldSet legend="Basic Information">
    <Field
      id="project-name"
      label="Name"
      error={errors?.name ?? null}
      validationState={validationState(errors?.name)}
      required
      let:describedBy
      let:validationState={nameValidationState}
    >
      <TextInput
        id="project-name"
        name="name"
        value={nameValue}
        describedBy={describedBy}
        validationState={nameValidationState}
        placeholder="e.g., Website Redesign"
        maxLength={128}
        on:valueChange={(event) => { nameValue = event.detail.value; }}
      />
    </Field>

    <Field
      id="project-description"
      label="Description"
      error={errors?.description ?? null}
      validationState={validationState(errors?.description)}
      let:describedBy
      let:validationState={descriptionValidationState}
    >
      <TextArea
        id="project-description"
        name="description"
        value={descriptionValue}
        describedBy={describedBy}
        validationState={descriptionValidationState}
        placeholder="Optional description for this project"
        rows={4}
        on:valueChange={(event) => { descriptionValue = event.detail.value; }}
      />
    </Field>
</FieldSet>

<FormActions align="start">
  <div class="project-form__actions" bind:this={actionBarElement}>
    <input type="hidden" name="intent" value={intent} />

    {#if returnTo}
      <input type="hidden" name="returnTo" value={returnTo} />
    {/if}

    <Button type="button" variant="ghost" on:click={handleCancel}>
      Cancel
    </Button>

    <SplitButton
      variant="primary"
      items={mode === "create" ? createIntentItems : editIntentItems}
      disabled={!isFormValid}
      on:click={() => submitWithIntent(intent)}
      on:action={(event) => submitWithIntent(event.detail.value as "save" | "save-close")}
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
  .project-form__actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--poodle-space-inline-md);
  }

  .inline-form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1rem;
    border: 1px solid var(--underlay-color-border-subtle, #e5e7eb);
    border-radius: 0.5rem;
    background: var(--underlay-color-surface-subtle, #f9fafb);
  }

  .inline-form-actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
  }
</style>
