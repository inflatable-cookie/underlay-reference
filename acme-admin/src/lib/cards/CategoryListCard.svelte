<script lang="ts">
  import { ListCard, Badge, DropdownMenu, DropdownItem, ConfirmAction } from "@decodelabs/underlay/components";
  import { gotoWithContext } from "@decodelabs/underlay/client";
  import type { CategoryWithCounts } from "@acme/client";
  import FolderOpen from "lucide-svelte/icons/folder-open";
  import Pencil from "lucide-svelte/icons/pencil";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import MoreVertical from "lucide-svelte/icons/more-vertical";

  interface Props {
    category: CategoryWithCounts;
    onDelete?: (categoryId: string) => void;
  }

  let { category, onDelete }: Props = $props();

  function handleEdit() {
    void gotoWithContext(`/categories/${category.id}/edit`, {
      label: "Categories",
      href: "/categories",
      type: "list"
    });
  }

  function handleDelete() {
    onDelete?.(category.id);
  }
</script>

<ListCard
  title={category.name}
  href={`/categories/${category.id}`}
  variant="standard"
  accent={category.color ?? "#6366f1"}
>
  {#snippet media()}
    <div class="icon" style:background={category.color ?? "#6366f1"}>
      <FolderOpen size={20} />
    </div>
  {/snippet}

  {#snippet badges()}
    {#if !category.isActive}
      <Badge variant="danger" size="sm">Inactive</Badge>
    {/if}
  {/snippet}

  {#snippet meta()}
    <span class="meta-item">{category.projectCount} projects</span>
    {#if category.description}
      <span class="meta-item">{category.description}</span>
    {/if}
  {/snippet}

  {#snippet actions()}
    <DropdownMenu>
      {#snippet trigger()}
        <button type="button" class="action-btn" aria-label="More actions">
          <MoreVertical size={16} />
        </button>
      {/snippet}
      <DropdownItem onclick={handleEdit}>
        <Pencil size={14} />
        Edit
      </DropdownItem>
      {#if onDelete}
        <ConfirmAction
          title="Delete Category"
          message={`Are you sure you want to delete "${category.name}"? Projects will be unassigned from this category.`}
          confirmLabel="Delete"
          onConfirm={handleDelete}
        >
          {#snippet trigger(triggerFn)}
            <DropdownItem onclick={triggerFn} variant="danger">
              <Trash2 size={14} />
              Delete
            </DropdownItem>
          {/snippet}
        </ConfirmAction>
      {/if}
    </DropdownMenu>
  {/snippet}
</ListCard>

<style>
  .icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2.5rem;
    height: 2.5rem;
    border-radius: 0.5rem;
    color: white;
  }

  .meta-item {
    display: inline-block;
    font-size: 0.875rem;
    color: var(--text-secondary, #6b7280);
  }

  .meta-item + .meta-item::before {
    content: "·";
    margin: 0 0.5rem;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: 0.375rem;
    cursor: pointer;
    color: var(--text-secondary, #6b7280);
  }

  .action-btn:hover {
    background: var(--bg-hover, #f3f4f6);
    color: var(--text-primary, #111827);
  }
</style>
