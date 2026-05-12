<script lang="ts">
  import { AlertDialog as PoodleAlertDialog } from "@poodle/svelte";
  import { EntityListCard, type EntityListCardBadge, type EntityListCardCounter } from "@decodelabs/underlay/templates";
  import type { ProjectWithCounts } from "@api-client";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";

  interface Props {
    project: ProjectWithCounts;
    onDelete?: (projectId: string) => void;
    reorderMode?: boolean;
    selectionMode?: boolean;
    selected?: boolean;
    onSelectionChange?: (projectId: string, selected: boolean) => void;
  }

  let { project, onDelete, reorderMode = false, selectionMode = false, selected = false, onSelectionChange }: Props = $props();

  let confirmDeleteOpen = $state(false);

  const progress = $derived(
    project.taskCount > 0
      ? Math.round((project.completedTaskCount / project.taskCount) * 100)
      : 0
  );

  const badges = $derived<EntityListCardBadge[]>([
    ...(project.categoryName ? [{ label: project.categoryName }] : [])
  ]);

  const counters = $derived<EntityListCardCounter[]>([
    {
      icon: "list-todo",
      count: project.taskCount,
      tooltip: `${project.taskCount} total task${project.taskCount === 1 ? "" : "s"}`
    },
    {
      icon: "check-check",
      count: project.completedTaskCount,
      tooltip: `${project.completedTaskCount} completed task${project.completedTaskCount === 1 ? "" : "s"}`
    }
  ]);

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
    void gotoWithContext(`/projects/${project.id}`, {
      label: "Projects",
      href: "/projects",
      type: "list"
    });
  }

  function handleEdit(): void {
    void gotoWithContext(`/projects/${project.id}/edit`, {
      label: "Projects",
      href: "/projects",
      type: "list"
    });
  }

  function handleDelete(): void {
    onDelete?.(project.id);
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

{#snippet progressFooter()}
  <span class="project-list-card__progress-label">
    {project.completedTaskCount}/{project.taskCount} tasks
  </span>
  {#if project.taskCount > 0}
    <div class="project-list-card__progress-bar">
      <div class="project-list-card__progress-fill" style:width="{progress}%"></div>
    </div>
  {/if}
{/snippet}

<EntityListCard
  title={project.name}
  leadingIcon="briefcase-business"
  notLive={project.status === "archived"}
  {reorderMode}
  reorderDisplay={{
    layout: "compact",
    showBadges: true,
    showFooter: false,
    showCounters: false,
    showSubtitle: false
  }}
  selectionMode={selectionMode}
  {selected}
  badges={badges}
  counters={counters}
  footer={progressFooter}
  contextMenuItems={selectionMode || reorderMode ? [] : menuItems}
  contextMenuAriaLabel="Project actions"
  contextMenuTrigger="leading"
  onSelectionChange={(nextSelected) => onSelectionChange?.(project.id, nextSelected)}
  onContextAction={handleContextAction}
  onClick={selectionMode || reorderMode ? undefined : handleOpen}
/>

{#if confirmDeleteOpen}
  <PoodleAlertDialog
    open={confirmDeleteOpen}
    title="Delete Project"
    description={`Are you sure you want to delete "${project.name}"? All tasks within this project will also be deleted.`}
    confirmLabel="Delete"
    onConfirm={handleDelete}
    onCancel={() => {
      confirmDeleteOpen = false;
    }}
    tone="danger"
  />
{/if}

<style>
  .project-list-card__progress-label {
    font-size: 0.75rem;
    color: var(--poodle-color-text-secondary);
  }

  .project-list-card__progress-bar {
    width: 3.5rem;
    height: 0.375rem;
    background: color-mix(in srgb, var(--poodle-color-border-subtle) 72%, transparent);
    border-radius: 999px;
    overflow: hidden;
  }

  .project-list-card__progress-fill {
    height: 100%;
    background: var(--poodle-color-accent-base);
    border-radius: 999px;
    transition: width 0.2s ease;
  }

</style>
