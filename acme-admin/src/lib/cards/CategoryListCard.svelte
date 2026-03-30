<script lang="ts">
  import {
    AlertDialog as PoodleAlertDialog,
    IconButton as PoodleIconButton,
    ListCard as PoodleListCard,
    Menu as PoodleMenu,
    Pill as PoodlePill
  } from "@poodle/svelte-primitives";
  import type { MenuItem } from "@poodle/svelte-primitives";
  import { gotoWithContext } from "@decodelabs/underlay/client";
  import type { CategoryWithCounts } from "@api-client";
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

<PoodleListCard
  title={category.name}
  href={`/categories/${category.id}`}
  accentColor={category.color ?? "#6366f1"}
>
  <svelte:fragment slot="leading">
    <FolderOpen size={20} />
  </svelte:fragment>

  <svelte:fragment slot="trailing">
    {#if !category.isActive}
      <PoodlePill tone="danger" appearance="badge" size="lg">Inactive</PoodlePill>
    {/if}
  </svelte:fragment>

  <svelte:fragment slot="actions">
    <PoodleMenu items={menuItems} placement="bottom-end" ariaLabel="Category actions" on:action={(event) => handleMenuAction(event.detail.value)}>
      <PoodleIconButton slot="trigger" icon="ellipsis" ariaLabel="Category actions" variant="ghost" />
    </PoodleMenu>
  </svelte:fragment>

  <span slot="footer" class="meta">
    <span class="meta-item">{category.projectCount} projects</span>
    {#if category.description}
      <span class="meta-item">{category.description}</span>
    {/if}
  </span>
</PoodleListCard>

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
