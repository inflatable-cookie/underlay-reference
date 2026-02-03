<script lang="ts">
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";
  import { adminCommands, type Category } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import { useAuthenticatedData, PageHeader, useToasts, Banner } from "@decodelabs/underlay/patterns";
  import { Button, PageLoading, FormError, ConfirmAction, Badge } from "@decodelabs/underlay/components";
  import { gotoWithContext } from "@decodelabs/underlay/client";
  import Pencil from "lucide-svelte/icons/pencil";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  const toastStore = useToasts();

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
</script>

{#if pageData.loading}
  <PageLoading message="Loading category..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if category}
  <PageHeader
    title={category.name}
    backHref="/categories"
    backLabel="Back to categories"
  >
    {#snippet actions()}
      <Button type="button" variant="secondary" onclick={handleEdit}>
        <Pencil size={16} />
        Edit
      </Button>
      <ConfirmAction
        title="Delete Category"
        description={`Are you sure you want to delete "${category.name}"? Projects will be unassigned from this category.`}
        confirmLabel="Delete"
        triggerLabel="Delete"
        triggerVariant="danger"
        onConfirm={handleDelete}
      />
    {/snippet}
  </PageHeader>

  {#if !category.isActive}
    <Banner variant="warning" message="This category is inactive and won't appear in selection lists." />
  {/if}

  <div class="detail-grid">
    <section class="detail-section">
      <h2>Details</h2>
      <dl class="detail-list">
        <div class="detail-item">
          <dt>Slug</dt>
          <dd><code>{category.slug}</code></dd>
        </div>
        <div class="detail-item">
          <dt>Status</dt>
          <dd>
            {#if category.isActive}
              <Badge variant="success">Active</Badge>
            {:else}
              <Badge variant="danger">Inactive</Badge>
            {/if}
          </dd>
        </div>
        <div class="detail-item">
          <dt>Color</dt>
          <dd>
            <span class="color-swatch" style:background={category.color ?? "#6366f1"}></span>
            {category.color ?? "#6366f1"}
          </dd>
        </div>
        {#if category.description}
          <div class="detail-item full">
            <dt>Description</dt>
            <dd>{category.description}</dd>
          </div>
        {/if}
      </dl>
    </section>

    <section class="detail-section">
      <h2>Metadata</h2>
      <dl class="detail-list">
        <div class="detail-item">
          <dt>ID</dt>
          <dd><code>{category.id}</code></dd>
        </div>
        <div class="detail-item">
          <dt>Weight</dt>
          <dd>{category.weight}</dd>
        </div>
        <div class="detail-item">
          <dt>Created</dt>
          <dd>{new Date(category.createdAt).toLocaleString()}</dd>
        </div>
        <div class="detail-item">
          <dt>Updated</dt>
          <dd>{new Date(category.updatedAt).toLocaleString()}</dd>
        </div>
      </dl>
    </section>
  </div>
{:else}
  <FormError message="Category not found" />
{/if}

<style>
  .detail-grid {
    display: grid;
    gap: 2rem;
    margin-top: 1.5rem;
  }

  .detail-section {
    background: var(--bg-surface, #fff);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 0.5rem;
    padding: 1.5rem;
  }

  .detail-section h2 {
    margin: 0 0 1rem;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary, #111827);
  }

  .detail-list {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
    margin: 0;
  }

  .detail-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .detail-item.full {
    grid-column: span 2;
  }

  .detail-item dt {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-secondary, #6b7280);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .detail-item dd {
    margin: 0;
    font-size: 0.875rem;
    color: var(--text-primary, #111827);
  }

  code {
    font-family: monospace;
    font-size: 0.8em;
    background: var(--bg-muted, #f3f4f6);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }

  .color-swatch {
    display: inline-block;
    width: 1rem;
    height: 1rem;
    border-radius: 0.25rem;
    vertical-align: middle;
    margin-right: 0.5rem;
    border: 1px solid rgba(0, 0, 0, 0.1);
  }
</style>
