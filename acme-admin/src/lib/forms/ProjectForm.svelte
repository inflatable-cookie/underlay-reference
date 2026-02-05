<script lang="ts">
  import {
    Field,
    FieldSet,
    FormActions,
    FormValidationProvider,
    SaveSplitButton,
    Select,
    TextButton,
    TextInput,
    TextArea
  } from "@decodelabs/underlay/components";
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

  // Form validation state
  let isFormValid = $state(false);

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
</script>

<FormValidationProvider bind:isValid={isFormValid}>
  <FieldSet legend="Organisation" columns={2}>
    <Field label="Category" error={errors?.categoryId} hint="Optional: Organize projects into categories">
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
              <button type="submit" class="btn-primary" disabled={isCreatingCategory}>
                {isCreatingCategory ? "Creating..." : "Create"}
              </button>
              <button type="button" class="btn-secondary" onclick={createCategoryInlineCancelHandler(onCancel)}>
                Cancel
              </button>
            </div>
          </form>
        {/snippet}
      </RelationSelector>
    </Field>

    <Field label="Status" error={errors?.status}>
      <Select
        name="status"
        value={statusValue}
        onchange={(val) => { statusValue = val; }}
        items={statusItems}
        placeholder="Select status"
      />
    </Field>
  </FieldSet>

  <FieldSet legend="Basic Information" columns={1}>
    <Field label="Name" error={errors?.name} required>
      <TextInput
        name="name"
        bind:value={nameValue}
        required
        placeholder="e.g., Website Redesign"
        maxlength={128}
      />
    </Field>

    <Field label="Description" error={errors?.description}>
      <TextArea
        name="description"
        bind:value={descriptionValue}
        placeholder="Optional description for this project"
        rows={4}
        maxlength={1000}
      />
    </Field>
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

  .btn-primary {
    padding: 0.5rem 1rem;
    background: var(--color-primary, #6366f1);
    color: white;
    border: none;
    border-radius: 0.375rem;
    cursor: pointer;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    padding: 0.5rem 1rem;
    background: var(--bg-secondary, #f3f4f6);
    color: var(--text-primary, #111827);
    border: 1px solid var(--underlay-color-border-subtle, #e5e7eb);
    border-radius: 0.375rem;
    cursor: pointer;
  }
</style>
