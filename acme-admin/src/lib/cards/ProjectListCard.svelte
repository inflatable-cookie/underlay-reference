<script lang="ts">
  import { ListCard } from "@decodelabs/underlay/components";
  import {
    AlertDialog as PoodleAlertDialog,
    Menu as PoodleMenu,
    Pill as PoodlePill
  } from "@poodle/svelte-primitives";
  import type { MenuItem } from "@poodle/svelte-primitives";
  import { gotoWithContext } from "@decodelabs/underlay/client";
  import type { ProjectWithCounts } from "acme-client";
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

  function handleCheckboxChange(event: Event) {
    const target = event.target as HTMLInputElement;
    onSelectionChange?.(project.id, target.checked);
  }

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

<ListCard title={project.name} href={selectionMode ? undefined : `/projects/${project.id}`}>
  {#snippet media()}
    {#if selectionMode}
      <label class="selection-checkbox">
        <input
          type="checkbox"
          checked={selected}
          onchange={handleCheckboxChange}
        />
      </label>
    {:else}
      <Briefcase size={30} />
    {/if}
  {/snippet}

  {#snippet trailing()}
    <PoodlePill tone={statusTone} appearance="badge" size="lg">{statusLabel}</PoodlePill>
    {#if project.categoryName}
      <PoodlePill tone="neutral" appearance="badge" size="lg">{project.categoryName}</PoodlePill>
    {/if}
  {/snippet}

  {#snippet actions({ trigger: mediaContent, align })}
    <PoodleMenu items={menuItems} placement={align === "end" ? "bottom-end" : "bottom-start"} on:action={(event) => handleMenuAction(event.detail.value)}>
      <div slot="trigger">
        {@render mediaContent()}
      </div>
    </PoodleMenu>
  {/snippet}

  <div class="task-progress">
    <span class="progress-label">{project.completedTaskCount}/{project.taskCount} tasks</span>
    {#if project.taskCount > 0}
      <div class="progress-bar">
        <div class="progress-bar__fill" style:width="{progress}%"></div>
      </div>
    {/if}
  </div>
</ListCard>

<PoodleAlertDialog
  bind:open={confirmDeleteOpen}
  title="Delete Project"
  description={`Are you sure you want to delete "${project.name}"? All tasks within this project will also be deleted.`}
  confirmLabel="Delete"
  onConfirm={handleDelete}
  tone="danger"
/>

<style>
  .selection-checkbox {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2.5rem;
    height: 2.5rem;
    cursor: pointer;
  }

  .selection-checkbox input[type="checkbox"] {
    width: 1.25rem;
    height: 1.25rem;
    accent-color: var(--accent-color, #6366f1);
    cursor: pointer;
  }

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
