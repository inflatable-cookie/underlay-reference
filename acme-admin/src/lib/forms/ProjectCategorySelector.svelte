<script lang="ts">
import type {
  SearchResult,
  SelectableRelation,
  SuggestionOptions,
  SelectionHistory
} from "@decodelabs/underlay/runtime";
import {
  onMount } from "svelte";
  import {
    Button,
  Callout,
  Dialog,
  SearchField
  } from "@poodle/svelte-primitives";
  import CategoryForm from "./CategoryForm.svelte";

  interface Props {
    value?: string | null;
    search: (query: string) => Promise<SearchResult<SelectableRelation>>;
    suggestions?: (options?: SuggestionOptions) => Promise<SelectableRelation[]>;
    selectionHistory?: SelectionHistory;
    placeholder?: string;
    label?: string;
    required?: boolean;
    disabled?: boolean;
    error?: string | null;
    createLabel?: string;
    createCategory?: (
      name: string,
      slug: string,
      description: string | null,
      color: string | null
    ) => Promise<SelectableRelation>;
  }

  let {
    value = $bindable(null),
    search,
    suggestions = undefined,
    selectionHistory = undefined,
    placeholder = "Select a category…",
    label = "Select Category",
    required = false,
    disabled = false,
    error = null,
    createLabel = "Add new category",
    createCategory = undefined
  }: Props = $props();

  let open = $state(false);
  let createMode = $state(false);
  let searchQuery = $state("");
  let searchResults = $state<SelectableRelation[]>([]);
  let suggestionItems = $state<SelectableRelation[]>([]);
  let selectedItem = $state<SelectableRelation | null>(null);
  let isSearching = $state(false);
  let isSuggestionsLoading = $state(false);
  let searchError = $state<string | null>(null);
  let createError = $state<string | null>(null);
  let isCreating = $state(false);
  let searchDebounceTimer = $state<ReturnType<typeof setTimeout> | null>(null);

  const hasSelection = $derived(selectedItem !== null);
  const displayItems = $derived(searchQuery.trim() ? searchResults : suggestionItems);
  const showClearButton = $derived(!required && hasSelection && !disabled);
  const showEmpty = $derived(!isSearching && !isSuggestionsLoading && !createMode && displayItems.length === 0);

  function rememberItem(item: SelectableRelation) {
    if (value && item.id === value) {
      selectedItem = item;
    }
  }

  async function loadSuggestions() {
    if (!suggestions) return;
    isSuggestionsLoading = true;
    searchError = null;

    try {
      const items = await suggestions({
        recentHints: selectionHistory?.getRecentIds()
      });
      suggestionItems = items;
      if (value) {
        const match = items.find((item) => item.id === value) ?? null;
        if (match) selectedItem = match;
      }
    } catch (e) {
      searchError = e instanceof Error ? e.message : "Failed to load suggestions";
    } finally {
      isSuggestionsLoading = false;
    }
  }

  async function performSearch(query: string) {
    const trimmed = query.trim();
    if (!trimmed) {
      searchResults = [];
      searchError = null;
      return;
    }

    isSearching = true;
    searchError = null;

    try {
      const result = await search(trimmed);
      searchResults = result.items;
      if (value) {
        const match = result.items.find((item) => item.id === value) ?? null;
        if (match) selectedItem = match;
      }
    } catch (e) {
      searchError = e instanceof Error ? e.message : "Failed to search categories";
      searchResults = [];
    } finally {
      isSearching = false;
    }
  }

  function scheduleSearch(query: string) {
    searchQuery = query;
    if (searchDebounceTimer) {
      clearTimeout(searchDebounceTimer);
      searchDebounceTimer = null;
    }

    if (!query.trim()) {
      searchResults = [];
      searchError = null;
      if (open && suggestions && suggestionItems.length === 0) {
        void loadSuggestions();
      }
      return;
    }

    searchDebounceTimer = setTimeout(() => {
      void performSearch(query);
    }, 300);
  }

  function handleOpen() {
    if (disabled) return;
    open = true;
    createMode = false;
    createError = null;
    if (suggestions && suggestionItems.length === 0) {
      void loadSuggestions();
    }
  }

  function handleClose() {
    open = false;
    createMode = false;
    createError = null;
    searchQuery = "";
    searchResults = [];
    searchError = null;
  }

  function handleSelect(item: SelectableRelation) {
    value = item.id;
    selectedItem = item;
    selectionHistory?.track(item.id);
    handleClose();
  }

  function handleClear() {
    value = null;
    selectedItem = null;
  }

  async function handleCreateSubmit(formData: FormData) {
    if (!createCategory) return;

    const name = String(formData.get("name") ?? "").trim();
    const slug = String(formData.get("slug") ?? "").trim();
    const description = String(formData.get("description") ?? "").trim() || null;
    const color = String(formData.get("color") ?? "#6366f1").trim() || null;

    if (!name || !slug) {
      createError = "Name and slug are required";
      return;
    }

    isCreating = true;
    createError = null;

    try {
      const item = await createCategory(name, slug, description, color);
      suggestionItems = [item, ...suggestionItems.filter((existing) => existing.id !== item.id)];
      handleSelect(item);
    } catch (e) {
      createError = e instanceof Error ? e.message : "Failed to create category";
    } finally {
      isCreating = false;
    }
  }

  $effect(() => {
    if (!value) {
      selectedItem = null;
    } else if (selectedItem?.id !== value) {
      const items = [...suggestionItems, ...searchResults];
      const match = items.find((item) => item.id === value) ?? null;
      if (match) selectedItem = match;
    }
  });

  onMount(() => {
    if (value && suggestions) {
      void loadSuggestions();
    }

    return () => {
      if (searchDebounceTimer) {
        clearTimeout(searchDebounceTimer);
      }
    };
  });
</script>

<div class="project-category-selector" class:project-category-selector--error={!!error}>
  <div class="project-category-selector__row">
    <button
      type="button"
      class="project-category-selector__trigger"
      class:project-category-selector__trigger--placeholder={!hasSelection}
      disabled={disabled}
      onclick={handleOpen}
    >
      <span class="project-category-selector__trigger-text">
        {selectedItem?.label ?? placeholder}
      </span>
      <span class="project-category-selector__trigger-actions">
        <span class="project-category-selector__chevron">▾</span>
      </span>
    </button>

    {#if showClearButton}
      <button
        type="button"
        class="project-category-selector__clear-button"
        onclick={handleClear}
      >
        Clear
      </button>
    {/if}
  </div>

  {#if error}
    <div class="project-category-selector__error">{error}</div>
  {/if}
</div>

<Dialog
  open={open}
  title={createMode ? createLabel : label}
  showCloseButton
  closeLabel={`Close ${label}`}
  contentClassName="project-category-selector__dialog"
  on:openChange={(event) => {
    open = event.detail.open;
    if (!event.detail.open) handleClose();
  }}
>
  {#if createMode}
    <form
      class="project-category-selector__create-form"
      onsubmit={(event) => {
        event.preventDefault();
        const formData = new FormData(event.currentTarget);
        void handleCreateSubmit(formData);
      }}
    >
      <CategoryForm
        mode="create"
        errors={createError ? { name: createError } : null}
      />

      <div class="project-category-selector__dialog-actions">
        <Button type="button" variant="ghost" on:click={() => {
          createMode = false;
          createError = null;
        }}>
          Back
        </Button>
        <Button type="submit" variant="primary" disabled={isCreating}>
          {isCreating ? "Creating..." : "Create"}
        </Button>
      </div>
    </form>
  {:else}
    <div class="project-category-selector__body">
      <div class="project-category-selector__toolbar">
        <SearchField
          id="project-category-search"
          value={searchQuery}
          placeholder="Search categories..."
          ariaLabel="Search categories"
          debounce={300}
          on:valueChange={(event) => scheduleSearch(event.detail.value)}
        />

        {#if createCategory}
          <Button type="button" variant="secondary" on:click={() => {
            createMode = true;
            createError = null;
          }}>
            {createLabel}
          </Button>
        {/if}
      </div>

      {#if searchError}
        <Callout tone="danger" message={searchError} announceMode="polite" />
      {/if}

      {#if isSearching || isSuggestionsLoading}
        <p class="project-category-selector__status">Loading…</p>
      {:else if showEmpty}
        <p class="project-category-selector__status">No categories found.</p>
      {:else}
        <ul class="project-category-selector__list">
          {#each displayItems as item (item.id)}
            <li>
              <button
                type="button"
                class="project-category-selector__option"
                disabled={item.disabled}
                onclick={() => handleSelect(item)}
              >
                <span class="project-category-selector__option-main">
                  <span class="project-category-selector__option-label">{item.label}</span>
                  {#if item.description}
                    <span class="project-category-selector__option-description">{item.description}</span>
                  {/if}
                </span>
                {#if value === item.id}
                  <span class="project-category-selector__option-selected">Selected</span>
                {/if}
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</Dialog>

<style>
  .project-category-selector {
    display: grid;
    gap: 0.35rem;
  }

  .project-category-selector__row {
    display: flex;
    gap: 0.5rem;
    align-items: stretch;
  }

  .project-category-selector__trigger {
    width: 100%;
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.625rem 0.75rem;
    border: 1px solid var(--poodle-color-border-default);
    border-radius: var(--poodle-radius-control);
    background: var(--poodle-color-background-input);
    color: var(--poodle-color-text-primary);
    text-align: left;
    cursor: pointer;
  }

  .project-category-selector__trigger--placeholder {
    color: var(--poodle-color-text-secondary);
  }

  .project-category-selector--error .project-category-selector__trigger {
    border-color: var(--poodle-color-status-danger-border);
  }

  .project-category-selector__trigger-text {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .project-category-selector__trigger-actions {
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
  }

  .project-category-selector__clear-button {
    padding: 0.625rem 0.75rem;
    border: 1px solid var(--poodle-color-border-default);
    border-radius: var(--poodle-radius-control);
    background: var(--poodle-color-background-surface);
    color: var(--poodle-color-text-secondary);
  }

  .project-category-selector__chevron {
    color: var(--poodle-color-text-secondary);
  }

  .project-category-selector__error {
    font-size: 0.85rem;
    color: var(--poodle-color-status-danger-text);
  }

  .project-category-selector__body {
    display: grid;
    gap: 0.875rem;
  }

  .project-category-selector__toolbar {
    display: grid;
    gap: 0.75rem;
  }

  .project-category-selector__list {
    display: grid;
    gap: 0.5rem;
    padding: 0;
    margin: 0;
    list-style: none;
  }

  .project-category-selector__option {
    width: 100%;
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.75rem;
    border: 1px solid var(--poodle-color-border-default);
    border-radius: var(--poodle-radius-surface);
    background: var(--poodle-color-background-surface);
    text-align: left;
    cursor: pointer;
  }

  .project-category-selector__option-main {
    display: grid;
    gap: 0.25rem;
    min-width: 0;
  }

  .project-category-selector__option-label {
    font-weight: 600;
  }

  .project-category-selector__option-description {
    color: var(--poodle-color-text-secondary);
    font-size: 0.9rem;
  }

  .project-category-selector__option-selected {
    color: var(--poodle-color-text-secondary);
    font-size: 0.82rem;
    white-space: nowrap;
  }

  .project-category-selector__status {
    margin: 0;
    color: var(--poodle-color-text-secondary);
  }

  .project-category-selector__create-form {
    display: grid;
    gap: 1rem;
  }

  .project-category-selector__dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
  }
</style>
