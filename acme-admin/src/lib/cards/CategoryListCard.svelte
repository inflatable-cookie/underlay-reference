<script lang="ts">
  import { AlertDialog as PoodleAlertDialog } from "@poodle/svelte";
  import { EntityListCard } from "@decodelabs/underlay/templates";
  import type { CategoryWithCounts } from "@api-client";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";

  interface Props {
    category: CategoryWithCounts;
    onDelete?: (categoryId: string) => void;
    reorderMode?: boolean;
    selectionMode?: boolean;
    selected?: boolean;
    onSelectionChange?: (categoryId: string, selected: boolean) => void;
  }

  let {
    category,
    onDelete,
    reorderMode = false,
    selectionMode = false,
    selected = false,
    onSelectionChange
  }: Props = $props();

  let confirmDeleteOpen = $state(false);

  const footerText = $derived(
    category.description
      ? `${category.projectCount} project${category.projectCount === 1 ? "" : "s"} · ${category.description}`
      : `${category.projectCount} project${category.projectCount === 1 ? "" : "s"}`
  );

  const menuItems = $derived([
    { value: "edit", label: "Edit" },
    ...(onDelete
      ? [
          { value: "separator", label: "", kind: "separator" as const },
          { value: "delete", label: "Delete", kind: "action" as const }
        ]
      : [])
  ]);

  function handleOpen(): void {
    void gotoWithContext(`/categories/${category.id}`, {
      label: "Categories",
      href: "/categories",
      type: "list"
    });
  }

  function handleEdit(): void {
    void gotoWithContext(`/categories/${category.id}/edit`, {
      label: "Categories",
      href: "/categories",
      type: "list"
    });
  }

  function handleDelete(): void {
    onDelete?.(category.id);
    confirmDeleteOpen = false;
  }

  function handleContextAction(value: string): void {
    if (value === "edit") {
      handleEdit();
      return;
    }

    if (value === "delete") {
      confirmDeleteOpen = true;
    }
  }
</script>

<EntityListCard
  title={category.name}
  {reorderMode}
  selectionMode={selectionMode}
  {selected}
  accentColor={category.color ?? "#6366f1"}
  notLive={!category.isActive}
  leadingIcon="folder-open"
  footerText={footerText}
  contextMenuItems={selectionMode || reorderMode ? [] : menuItems}
  contextMenuAriaLabel="Category actions"
  contextMenuTrigger="leading"
  onSelectionChange={(nextSelected) => onSelectionChange?.(category.id, nextSelected)}
  onContextAction={handleContextAction}
  onClick={selectionMode || reorderMode ? undefined : handleOpen}
/>

{#if confirmDeleteOpen}
  <PoodleAlertDialog
    open={confirmDeleteOpen}
    title="Delete Category"
    description={`Are you sure you want to delete "${category.name}"? Projects will be unassigned from this category.`}
    confirmLabel="Delete"
    onConfirm={handleDelete}
    onCancel={() => {
      confirmDeleteOpen = false;
    }}
    tone="danger"
  />
{/if}
