<script lang="ts">
import {
  useToasts
} from "@decodelabs/underlay/runtime/feedback";
import {
  useAuthenticatedData
} from "@decodelabs/underlay/runtime/auth";
import {
  AlertDialog as PoodleAlertDialog,
  Button as PoodleButton,
  Callout as PoodleCallout,
  Card as PoodleCard,
  Code as PoodleCode,
  DetailItem as PoodleDetailItem,
  Menu as PoodleMenu,
  type MenuItem
  } from "@poodle/svelte-primitives";
  import { DetailSection as PoodleDetailSection,
  PageHeader as PoodlePageHeader,
  PageLoading } from "@poodle/svelte-composites";
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";
  import { adminCommands,
  type Category } from "@api-client";
  import { copyToClipboard } from "@decodelabs/underlay/runtime/feedback";
  import { auth } from "$lib/stores/auth";
    import { MetaBar as PoodleMetaBar, MetaItem as PoodleMetaItem, Pill as PoodlePill, TimeAgo } from "@poodle/svelte-primitives";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";

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
  let showDeleteConfirm = $state(false);
  const actionItems: MenuItem[] = [
    { value: "edit", label: "Edit" },
    { value: "delete", label: "Delete", tone: "danger" },
    { value: "separator-copy", label: "", kind: "separator" },
    { value: "copy-id", label: "Copy ID" }
  ];

  function handleEdit() {
    if (!category) return;
    void gotoWithContext(`/categories/${category.id}/edit`, {
      label: category.name,
      href: `/categories/${category.id}`,
      type: "detail"
    });
  }

  async function handleAction(value: string) {
    if (!category) return;

    if (value === "edit") {
      handleEdit();
      return;
    }

    if (value === "delete") {
      showDeleteConfirm = true;
      return;
    }

    if (value === "copy-id") {
      await copyToClipboard(toastStore, category.id, "Copied category ID");
    }
  }
</script>

{#if pageData.loading}
  <PageLoading presentation="inline" message="Loading category..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if category}
  <section class="category-detail">
    <div class="category-detail__header">
      <PoodlePageHeader
        section="Category"
        title={category.name}
        backHref="/categories"
        backLabel="Back to categories"
        subtitle={category.description ?? undefined}
        bannerMessage={!category.isActive ? "This category is inactive and won't appear in selection lists." : undefined}
        bannerTone={!category.isActive ? "warning" : "warning"}
      >
        {#snippet actions()}
          <PoodleMenu
            items={actionItems}
            placement="bottom-end"
            triggerAriaLabel="Category actions"
            on:action={(event) => void handleAction(event.detail.value)}
          >
            <svelte:fragment slot="trigger">
              <PoodleButton variant="secondary">Actions</PoodleButton>
            </svelte:fragment>
          </PoodleMenu>
        {/snippet}
      </PoodlePageHeader>

      <div class="category-detail__meta">
      <PoodleMetaBar ariaLabel="Category metadata">
        <PoodleMetaItem label="ID">
          <PoodleCode inline source={category.id} showCopyButton />
        </PoodleMetaItem>
        <PoodlePill tone={category.isActive ? "success" : "neutral"} appearance="badge" size="lg">
          {category.isActive ? "Active" : "Inactive"}
        </PoodlePill>
      </PoodleMetaBar>
      </div>
    </div>

    <PoodleCard>
      <div class="detail-card-grid">
        <PoodleDetailSection title="Details" columns={2} separated={false}>
          <PoodleDetailItem presentation="surface" label="Slug">
            <svelte:fragment slot="value"><code>{category.slug}</code></svelte:fragment>
          </PoodleDetailItem>
          <PoodleDetailItem presentation="surface" label="Color">
            <svelte:fragment slot="value">
              <span class="color-value">
                <span class="color-swatch" style:background={category.color ?? "#6366f1"}></span>
                <span>{category.color ?? "#6366f1"}</span>
              </span>
            </svelte:fragment>
          </PoodleDetailItem>
        </PoodleDetailSection>

        <PoodleDetailSection title="Metadata" columns={2} separated={false}>
          <PoodleDetailItem presentation="surface" label="Created">
            <svelte:fragment slot="value">
              <TimeAgo datetime={category.createdAt} tooltipFormat="datetime" />
            </svelte:fragment>
          </PoodleDetailItem>
          <PoodleDetailItem presentation="surface" label="Updated">
            <svelte:fragment slot="value">
              <TimeAgo datetime={category.updatedAt} tooltipFormat="datetime" />
            </svelte:fragment>
          </PoodleDetailItem>
        </PoodleDetailSection>
      </div>
    </PoodleCard>
  </section>

  <PoodleAlertDialog
    open={showDeleteConfirm}
    title="Delete Category"
    description={`Are you sure you want to delete "${category.name}"? Projects will be unassigned from this category.`}
    confirmLabel="Delete"
    tone="danger"
    onConfirm={async () => {
      const token = auth.getToken();
      if (!token) throw new Error("Not authenticated");
      await adminCommands.softDeleteCategory(category.id, fetch, token);
      await goto("/categories");
    }}
    onCancel={() => {
      showDeleteConfirm = false;
    }}
  >
    <p><strong>{category.name}</strong></p>
  </PoodleAlertDialog>
{:else}
  <PoodleCallout tone="danger" message="Category not found" announceMode="polite" />
{/if}

<style>
  .category-detail {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .category-detail__header {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .category-detail__meta {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .detail-card-grid {
    display: grid;
    gap: 1rem;
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

  .color-value {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }
</style>
