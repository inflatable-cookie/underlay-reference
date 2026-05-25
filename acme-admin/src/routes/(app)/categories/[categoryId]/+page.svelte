<script lang="ts">
  import {
    EntityDetailPage,
    EntityDetail,
    EntityAttributeList,
    EntityDetailModule
  } from "@decodelabs/underlay/templates";
  import { useAuthenticatedData } from "@decodelabs/underlay/runtime/auth";
  import { copyToClipboard, useToasts } from "@decodelabs/underlay/runtime/feedback";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import { goto } from "$app/navigation";
  import { auth } from "$lib/stores/auth";
  import type { PageData } from "./$types";
  import { adminCommands, type Category } from "@api-client";
  import ProjectsListPage from "$lib/lists/ProjectsListPage.svelte";
  import { Code, Pill, TimeAgo } from "@poodle/svelte";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  const toastStore = useToasts();

  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const category = await adminCommands.getCategory(data.categoryId, fetch, token);
      return { category };
    },
    {
      defaultValue: { category: null as Category | null }
    }
  );

  const category = $derived(pageData.data?.category);

  async function categoryLoader(fetch: typeof window.fetch, token: string | null) {
    if (!token) throw new Error("Not authenticated");
    const nextCategory = await adminCommands.getCategory(data.categoryId, fetch, token);
    return nextCategory;
  }

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
    if (!token) throw new Error("Not authenticated");
    await adminCommands.softDeleteCategory(category.id, fetch, token);
    await goto("/categories");
  }

  async function handleCopyId() {
    if (!category) return;
    await copyToClipboard(toastStore, category.id, "Copied category ID");
  }

  const detailPageMeta = $derived.by(() => [
    { label: "ID", value: idSnippet as never },
    { label: "", value: statusSnippet as never, separator: false }
  ]);

  const detailPageTabs = $derived.by(() => [
    {
      id: "details",
      label: "Details",
      content: detailsTabSnippet as never
    },
    {
      id: "projects",
      label: "Projects",
      content: projectsTabSnippet as never
    }
  ]);

  const bannerMessage = $derived(
    category?.isActive ? undefined : "This category is inactive and won't appear in selection lists."
  );
</script>

<EntityDetailPage
  title={category?.name ?? "Category"}
  section="Categories"
  backHref="/categories"
  backLabel="Back to categories"
  bannerMessage={bannerMessage}
  bannerTone="warning"
  dataLoader={categoryLoader}
  meta={detailPageMeta as never}
  tabs={detailPageTabs as never}
  actions={[
    { label: "Edit", handler: handleEdit },
    {
      label: "Copy ID",
      handler: handleCopyId
    },
    {
      label: "Delete",
      tone: "danger",
      confirm: {
        title: "Delete category",
        description: "Are you sure you want to delete this category? Projects will be unassigned from it.",
        confirmLabel: "Delete category",
        cancelLabel: "Keep category"
      },
      handler: handleDelete
    }
  ]}
/>

{#snippet idSnippet()}
  {#if category}
    <Code
      inline
      inlineVariant="plain"
      typography="inline"
      source={category.id}
      showCopyButton
    />
  {/if}
{/snippet}

{#snippet statusSnippet()}
  {#if category}
    <Pill
      tone={category.isActive ? "success" : "neutral"}
      appearance="badge"
      size="sm"
      typography="inherit"
    >
      {category.isActive ? "Active" : "Inactive"}
    </Pill>
  {/if}
{/snippet}

{#snippet slugSnippet()}
  {#if category}
    <Code
      inline
      inlineVariant="plain"
      typography="inline"
      source={category.slug}
      showCopyButton={false}
    />
  {/if}
{/snippet}

{#snippet colorSnippet()}
  {#if category}
    <span class="color-value">
      <span class="color-swatch" style:background={category.color ?? "#6366f1"}></span>
      <span>{category.color ?? "#6366f1"}</span>
    </span>
  {/if}
{/snippet}

{#snippet createdSnippet()}
  {#if category}
    <TimeAgo datetime={category.createdAt} tooltipFormat="datetime" />
  {/if}
{/snippet}

{#snippet updatedSnippet()}
  {#if category}
    <TimeAgo datetime={category.updatedAt} tooltipFormat="datetime" />
  {/if}
{/snippet}

{#snippet detailsTabSnippet(loadedCategory: Category)}
  <EntityDetail>
    {#snippet children()}
      <EntityAttributeList
        title={null}
        items={[
          {
            label: "Slug",
            value: slugSnippet as never,
            layout: "stacked",
            presentation: "surface"
          },
          {
            label: "Color",
            value: colorSnippet as never,
            layout: "stacked",
            presentation: "surface"
          },
          {
            label: "Created",
            value: createdSnippet as never,
            layout: "stacked",
            presentation: "surface"
          },
          {
            label: "Updated",
            value: updatedSnippet as never,
            layout: "stacked",
            presentation: "surface"
          }
        ]}
      />

      {#if loadedCategory.description}
        <EntityDetailModule>
          {#snippet children()}
            <div class="category-detail-copy">
              {loadedCategory.description}
            </div>
          {/snippet}
        </EntityDetailModule>
      {/if}
    {/snippet}
  </EntityDetail>
{/snippet}

{#snippet projectsTabSnippet(loadedCategory: Category)}
  <ProjectsListPage
    title="Projects"
    hideTitle
    subtitle={`Projects in ${loadedCategory.name}`}
    headerLevel={3}
    categoryId={loadedCategory.id}
  />
{/snippet}

<style>
  .color-swatch {
    display: inline-block;
    width: 1rem;
    height: 1rem;
    border-radius: 0.25rem;
    border: 1px solid rgba(0, 0, 0, 0.1);
  }

  .color-value {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }

  .category-detail-copy {
    max-width: 42rem;
    color: var(--poodle-color-text-primary);
    font-size: var(--poodle-typography-body-size);
    line-height: 1.7;
    white-space: pre-wrap;
  }
</style>
