<script lang="ts">
  import { ListCard, Pill, DropdownMenu, AlertDialog } from "@decodelabs/underlay/components";
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

  const menuItems = $derived([
    { label: "Edit", onSelect: handleEdit },
    ...(onDelete
      ? [
          { separator: true },
          {
            label: "Delete",
            destructive: true,
            onSelect: () => (confirmDeleteOpen = true)
          }
        ]
      : [])
  ]);
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
      <Pill accent="#ef4444">Inactive</Pill>
    {/if}
  {/snippet}

  {#snippet actions({ trigger: mediaContent, align })}
    <DropdownMenu items={menuItems} {align}>
      {#snippet trigger()}
        {@render mediaContent()}
      {/snippet}
    </DropdownMenu>
  {/snippet}

  <span class="meta">
    <span class="meta-item">{category.projectCount} projects</span>
    {#if category.description}
      <span class="meta-item">{category.description}</span>
    {/if}
  </span>
</ListCard>

<AlertDialog
  bind:open={confirmDeleteOpen}
  showTrigger={false}
  title="Delete Category"
  description={`Are you sure you want to delete "${category.name}"? Projects will be unassigned from this category.`}
  confirmLabel="Delete"
  onConfirm={handleDelete}
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
