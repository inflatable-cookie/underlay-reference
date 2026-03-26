<script lang="ts">
  import { ListCard } from "@decodelabs/underlay/components";
  import {
    AlertDialog as PoodleAlertDialog,
    Menu as PoodleMenu,
    Pill as PoodlePill
  } from "@poodle/svelte-primitives";
  import type { MenuItem } from "@poodle/svelte-primitives";
  import { gotoWithContext } from "@decodelabs/underlay/client";
  import type { CategoryWithCounts } from "acme-client";
  import FolderOpen from "lucide-svelte/icons/folder-open";

  interface Props {
    category: CategoryWithCounts;
    onDelete?: (categoryId: string) => void;
  }

  let { category, onDelete }: Props = $props();

  let confirmDeleteOpen = $state(false);

  function handleEdit() {
    void gotoWithContext(`/categories/${category.id}/edit`, {
      label: "Categories",
      href: "/categories",
      type: "list"
    });
  }

  function handleDelete() {
    onDelete?.(category.id);
    confirmDeleteOpen = false;
  }

  const menuItems = $derived<MenuItem[]>([
    { value: "edit", label: "Edit" },
    ...(onDelete
      ? [
          { value: "separator", label: "", kind: "separator" as const },
          {
            value: "delete",
            label: "Delete",
            kind: "action" as const
          }
        ]
      : [])
  ]);

  function handleMenuAction(value: string) {
    if (value === "edit") {
      handleEdit();
      return;
    }

    if (value === "delete") {
      confirmDeleteOpen = true;
    }
  }
</script>

<ListCard
  title={category.name}
  href={`/categories/${category.id}`}
  accent={category.color ?? "#6366f1"}
>
  {#snippet media()}
    <FolderOpen size={30} />
  {/snippet}

  {#snippet trailing()}
    {#if !category.isActive}
      <PoodlePill tone="danger" appearance="badge" size="lg">Inactive</PoodlePill>
    {/if}
  {/snippet}

  {#snippet actions({ trigger: mediaContent, align })}
    <PoodleMenu items={menuItems} placement={align === "end" ? "bottom-end" : "bottom-start"} on:action={(event) => handleMenuAction(event.detail.value)}>
      <div slot="trigger">
        {@render mediaContent()}
      </div>
    </PoodleMenu>
  {/snippet}

  <span class="meta">
    <span class="meta-item">{category.projectCount} projects</span>
    {#if category.description}
      <span class="meta-item">{category.description}</span>
    {/if}
  </span>
</ListCard>

<PoodleAlertDialog
  bind:open={confirmDeleteOpen}
  title="Delete Category"
  description={`Are you sure you want to delete "${category.name}"? Projects will be unassigned from this category.`}
  confirmLabel="Delete"
  onConfirm={handleDelete}
  tone="danger"
/>

<style>
  .meta {
    font-size: 0.875rem;
    color: var(--text-secondary, #6b7280);
  }

  .meta-item + .meta-item::before {
    content: "·";
    margin: 0 0.5rem;
  }
</style>
