<script lang="ts">
import "@acme/ui/editor";
import "@acme/ui/validation";
import {
  createLocalSearchFns,
  type SearchResult,
  type SuggestionOptions
} from "@inflatable-cookie/underlay/runtime/relations";
import { NightfireEditor } from "@inflatable-cookie/underlay/nightfire/editor";
import {
  prepareNightfireForSave,
  type NightfireDraftValue,
  type NightfireValue
} from "@inflatable-cookie/underlay/nightfire/validation";
import {
  Button,
  Field,
  FieldSet,
  FormActions,
  SplitButton,
  Select,
  TextInput
  } from "@inflatable-cookie/poodle-svelte";
    import { navigateOnCancel } from "@inflatable-cookie/underlay/client/navigation";
  import { categorySelectionHistory } from "$lib/stores/selection-history";
  import { untrack } from "svelte";
  import type { Category, CategoryWithCounts, SuggestionRequestOptions } from "@api-client";
  import ProjectCategorySelector from "./ProjectCategorySelector.svelte";

  type ProjectFormMode = "create" | "edit";

  interface ProjectFormValues {
    name?: string;
    description?: NightfireValue | null;
    categoryId?: string | null;
    status?: string;
  }

  const PROJECT_DESCRIPTION_SCHEMA = "acme:project/description@1";

  /** Converts a Category to local category-selector item */
  function categoryToSelectable(category: Category | CategoryWithCounts) {
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
    fetchCategories?: (options?: SuggestionRequestOptions) => Promise<Category[]>;
    initialCategorySelection?: ReturnType<typeof categoryToSelectable> | null;
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
    initialCategorySelection = null,
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
  let descriptionValue = $state<NightfireDraftValue>(untrack(() => (
    isNightfireValue(values.description)
      ? values.description
      : { schema: PROJECT_DESCRIPTION_SCHEMA, blocks: [] }
  )));
  let categoryId = $state<string | null>(untrack(() => values.categoryId ?? null));
  let statusValue = $state(untrack(() => values.status ?? "active"));
  const serialisedDescription = $derived.by(() => {
    const prepared = prepareNightfireForSave(descriptionValue);
    return prepared ? JSON.stringify(prepared) : "";
  });

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

  // Search function for RelationSelector when using server-side search
  const searchCategoriesServer = async (query: string): Promise<SearchResult<ReturnType<typeof categoryToSelectable>>> => {
    if (!fetchCategories) {
      return searchCategories(query);
    }
    const cats = await fetchCategories({ query, limit: 20 });
    return { items: cats.map(categoryToSelectable), total: cats.length };
  };

  // Suggestions function
  const suggestCategoriesServer = async (options?: SuggestionOptions): Promise<ReturnType<typeof categoryToSelectable>[]> => {
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

  function isNightfireValue(value: unknown): value is NightfireValue {
    return !!value && typeof value === "object" && "schema" in value;
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
      <ProjectCategorySelector
        label="Select Category"
        search={fetchCategories ? searchCategoriesServer : searchCategories}
        suggestions={fetchCategories ? suggestCategoriesServer : suggestCategories}
        initialSelection={initialCategorySelection}
        selectionHistory={categorySelectionHistory}
        placeholder="Select a category…"
        createLabel="Add new category"
        createCategory={createCategory
          ? async (name, slug, description, color) => categoryToSelectable(
              await createCategory(name, slug, description, color)
            )
          : undefined}
        error={errors?.categoryId ?? null}
        required={false}
        bind:value={categoryId}
      />
    </Field>

    <Field
      id="project-status"
      label="Status"
      error={errors?.status ?? null}
      validationState={validationState(errors?.status)}
      required
    >
      {#snippet control({ describedBy })}
        <Select
          id="project-status"
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

<FieldSet legend="Basic Information">
    <Field
      id="project-name"
      label="Name"
      error={errors?.name ?? null}
      validationState={validationState(errors?.name)}
      required
    >
      {#snippet control({ describedBy, validationState })}
        <TextInput
          id="project-name"
          name="name"
          value={nameValue}
          describedBy={describedBy}
          validationState={validationState}
          placeholder="e.g., Website Redesign"
          maxLength={128}
          onValueChange={(nextValue) => { nameValue = nextValue; }}
        />
      {/snippet}
    </Field>

    <Field
      id="project-description"
      label="Description"
      error={errors?.description ?? null}
      validationState={validationState(errors?.description)}
    >
      <input type="hidden" name="description" value={serialisedDescription} />
      <NightfireEditor
        name="description"
        schema={PROJECT_DESCRIPTION_SCHEMA}
        bind:value={descriptionValue}
      />
    </Field>
</FieldSet>

<FormActions align="end" showTopBorder>
  <div class="project-form__actions" bind:this={actionBarElement}>
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
  .project-form__actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--poodle-space-inline-md);
  }
</style>
