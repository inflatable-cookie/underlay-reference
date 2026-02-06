<script lang="ts">
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";
  import { adminCommands, type Category } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import {
    useAuthenticatedData,
    PageHeader,
    PageHeaderMeta,
    PageHeaderMetaRow,
    PageHeaderMetaItem,
    PageHeaderMetaSeparator,
    useToasts,
    Banner
  } from "@decodelabs/underlay/patterns";
  import { Code, PageLoading, FormError, Badge, Pill, DetailsCard, DetailsSection, DetailsItem, TimeAgo, DropdownMenu, AlertDialog } from "@decodelabs/underlay/components";
  import { gotoWithContext } from "@decodelabs/underlay/client";
  import MoreVertical from "lucide-svelte/icons/more-vertical";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  const toastStore = useToasts();
  let confirmDeleteOpen = $state(false);

  // Fetch category data
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const category = await adminCommands.getCategory(data.categoryId, fetch, token);
      return { category };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { category: null as Category | null }
    }
  );

  // Trigger fetch when auth is ready
  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  const category = $derived(pageData.data?.category);

  function handleEdit() {
    if (!category) return;
    void gotoWithContext(`/categories/${category.id}/edit`, {
      label: category.name,
      href: `/categories/${category.id}`,
      type: "detail"
    });
  }

  async function handleDelete() {
    if (!category) return;

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await adminCommands.softDeleteCategory(category.id, fetch, token);
      toastStore.push({ variant: "success", message: "Category deleted" });
      await goto("/categories");
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete category";
      toastStore.push({ variant: "error", message });
    }
  }

  const menuItems = $derived([
    { label: "Edit", onSelect: handleEdit },
    { separator: true },
    {
      label: "Delete",
      destructive: true,
      onSelect: () => (confirmDeleteOpen = true)
    }
  ]);
</script>

{#if pageData.loading}
  <PageLoading message="Loading category..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if category}
  <PageHeader
    section="Category"
    title={category.name}
    backHref="/categories"
    backLabel="Back to categories"
    subtitle={category.description ?? undefined}
  >
    <PageHeaderMeta>
      <PageHeaderMetaRow>
        <PageHeaderMetaItem label="ID">
          <Code copy>{category.id}</Code>
        </PageHeaderMetaItem>
        <PageHeaderMetaSeparator />
        <Pill accent={category.isActive ? "#10b981" : "#6b7280"}>
          {category.isActive ? "Active" : "Inactive"}
        </Pill>
      </PageHeaderMetaRow>
    </PageHeaderMeta>

    {#snippet actions()}
      <DropdownMenu items={menuItems} triggerAriaLabel="Category actions">
        {#snippet trigger()}
          <MoreVertical size={16} aria-hidden="true" />
        {/snippet}
      </DropdownMenu>
    {/snippet}
  </PageHeader>

  <AlertDialog
    bind:open={confirmDeleteOpen}
    showTrigger={false}
    title="Delete Category"
    description={`Are you sure you want to delete "${category.name}"? Projects will be unassigned from this category.`}
    confirmLabel="Delete"
    onConfirm={handleDelete}
  />

  {#if !category.isActive}
    <div class="category-banner">
      <Banner variant="warning" message="This category is inactive and won't appear in selection lists." />
    </div>
  {/if}

  <DetailsCard>
    <DetailsSection legend="Details">
      <DetailsItem label="Slug" value={category.slug} code />
      <DetailsItem label="Color">
        <span class="color-swatch" style:background={category.color ?? "#6366f1"}></span>
        <span class="color-value">{category.color ?? "#6366f1"}</span>
      </DetailsItem>
    </DetailsSection>

    <DetailsSection legend="Metadata">
      <DetailsItem label="Created">
        <TimeAgo date={category.createdAt} tooltipFormat="datetime" />
      </DetailsItem>
      <DetailsItem label="Updated">
        <TimeAgo date={category.updatedAt} tooltipFormat="datetime" />
      </DetailsItem>
    </DetailsSection>
  </DetailsCard>
{:else}
  <FormError message="Category not found" />
{/if}

<style>
  .color-swatch {
    display: inline-block;
    width: 1rem;
    height: 1rem;
    border-radius: 0.25rem;
    vertical-align: middle;
    margin-right: 0.5rem;
    border: 1px solid rgba(0, 0, 0, 0.1);
  }

  .color-value {
    vertical-align: middle;
  }

  .category-banner {
    margin-bottom: 1rem;
  }
</style>
