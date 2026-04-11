<script lang="ts">
  import {
    AlertDialog as PoodleAlertDialog,
    IconButton as PoodleIconButton,
    ListCard as PoodleListCard,
    Menu as PoodleMenu,
    Pill as PoodlePill
  } from "@poodle/svelte";
  import type { MenuItem } from "@poodle/svelte";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import type { ProjectWithCounts } from "@api-client";
  import Briefcase from "lucide-svelte/icons/briefcase";

  interface Props {
    project: ProjectWithCounts;
    onDelete?: (projectId: string) => void;
    /** Whether selection mode is active */
    selectionMode?: boolean;
    /** Whether this item is selected */
    selected?: boolean;
    /** Callback when selection changes */
    onSelectionChange?: (projectId: string, selected: boolean) => void;
  }

  let { project, onDelete, selectionMode = false, selected = false, onSelectionChange }: Props = $props();

  let confirmDeleteOpen = $state(false);

  const progress = $derived(
    project.taskCount > 0
      ? Math.round((project.completedTaskCount / project.taskCount) * 100)
      : 0
  );

  const statusLabel = $derived(
    ({ active: "Active", archived: "Archived", on_hold: "On Hold" } as Record<string, string>)[
      project.status
    ] ?? project.status
  );

  const statusTone = $derived(
    project.status === "active" ? "success" : "neutral"
  );

  function handleEdit() {
    void gotoWithContext(`/projects/${project.id}/edit`, {
      label: "Projects",
      href: "/projects",
      type: "list"
    });
  }

  function handleDelete() {
    onDelete?.(project.id);
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
  title={project.name}
  href={selectionMode ? undefined : `/projects/${project.id}`}
  selectable={selectionMode}
  {selected}
  on:selectedChange={(event) => onSelectionChange?.(project.id, event.detail.selected)}
>
  <svelte:fragment slot="leading">
    <Briefcase size={20} />
  </svelte:fragment>

  <svelte:fragment slot="trailing">
    <PoodlePill tone={statusTone} appearance="badge" size="lg">{statusLabel}</PoodlePill>
    {#if project.categoryName}
      <PoodlePill tone="neutral" appearance="badge" size="lg">{project.categoryName}</PoodlePill>
    {/if}
  </svelte:fragment>

  <svelte:fragment slot="actions">
    {#if !selectionMode}
      <PoodleMenu items={menuItems} placement="bottom-end" ariaLabel="Project actions" on:action={(event) => handleMenuAction(event.detail.value)}>
        <PoodleIconButton slot="trigger" icon="ellipsis" ariaLabel="Project actions" variant="ghost" />
      </PoodleMenu>
    {/if}
  </svelte:fragment>

  <div slot="footer" class="task-progress">
    <span class="progress-label">{project.completedTaskCount}/{project.taskCount} tasks</span>
    {#if project.taskCount > 0}
      <div class="progress-bar">
        <div class="progress-bar__fill" style:width="{progress}%"></div>
      </div>
    {/if}
  </div>
</PoodleListCard>

<PoodleAlertDialog
  bind:open={confirmDeleteOpen}
  title="Delete Project"
  description={`Are you sure you want to delete "${project.name}"? All tasks within this project will also be deleted.`}
  confirmLabel="Delete"
  onConfirm={handleDelete}
  tone="danger"
/>

<style>
  .task-progress {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .progress-label {
    font-size: 0.75rem;
    color: var(--text-secondary, #6b7280);
  }

  .progress-bar {
    height: 0.375rem;
    background: var(--underlay-color-surface-hover, #e5e7eb);
    border-radius: 0.25rem;
    overflow: hidden;
  }

  .progress-bar__fill {
    height: 100%;
    background: var(--underlay-color-accent, #6366f1);
    border-radius: 0.25rem;
    transition: width 0.2s ease;
  }
</style>
