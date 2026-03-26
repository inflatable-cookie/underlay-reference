<script lang="ts">
  import { Callout as PoodleCallout } from "@poodle/svelte-primitives";
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";
  import { adminCommands, type Category } from "acme-client";
  import { auth } from "$lib/stores/auth";
  import {
    useAuthenticatedData,
    useToasts,
    EntityActionsMenu,
    DetailPageShell,
    DetailMeta,
    DetailMetaId,
    DetailMetaStatus,
    DetailMetaSeparator
  } from "@decodelabs/underlay/patterns";
  import { PageLoading, DetailsCard, DetailsSection, DetailsItem, TimeAgo } from "@decodelabs/underlay/components";
  import { gotoWithContext } from "@decodelabs/underlay/client";

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
      defaultValue: { category: null as Category | null }
    }
  );

  const category = $derived(pageData.data?.category);

  function handleEdit() {
    if (!category) return;
    void gotoWithContext(`/categories/${category.id}/edit`, {
      label: category.name,
      href: `/categories/${category.id}`,
      type: "detail"
    });
  }
</script>

{#if pageData.loading}
  <PageLoading message="Loading category..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if category}
  <DetailPageShell
    section="Category"
    title={category.name}
    backHref="/categories"
    backLabel="Back to categories"
    subtitle={category.description ?? undefined}
    bannerMessage={!category.isActive ? "This category is inactive and won't appear in selection lists." : undefined}
    bannerVariant={!category.isActive ? "warning" : undefined}
  >
    {#snippet meta()}
      <DetailMeta>
        <DetailMetaId value={category.id} />
        <DetailMetaSeparator />
        <DetailMetaStatus value={category.isActive} trueLabel="Active" falseLabel="Inactive" />
      </DetailMeta>
    {/snippet}

    {#snippet actions()}
      <EntityActionsMenu
        toastStore={toastStore}
        copies={[{ label: "Copy ID", text: category.id, successMessage: "Copied category ID" }]}
        onEdit={handleEdit}
        deleteConfig={{
          entityLabel: category.name,
          title: "Delete Category",
          description: `Are you sure you want to delete "${category.name}"? Projects will be unassigned from this category.`,
          confirmLabel: "Delete",
          execute: async () => {
            const token = auth.getToken();
            if (!token) throw new Error("Not authenticated");
            await adminCommands.softDeleteCategory(category.id, fetch, token);
          }
        }}
        onDeleteSuccess={() => goto("/categories")}
      />
    {/snippet}

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
  </DetailPageShell>
{:else}
  <PoodleCallout tone="danger" message="Category not found" announceMode="polite" />
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
</style>
