<script lang="ts">
  import { ListCard, Badge, DropdownMenu, AlertDialog } from "@decodelabs/underlay/components";
  import { gotoWithContext } from "@decodelabs/underlay/client";
  import type { ProjectWithCounts } from "acme-client";
  import Briefcase from "lucide-svelte/icons/briefcase";

  interface Props {
    project: ProjectWithCounts;
    onDelete?: (projectId: string) => void;
  }

  let { project, onDelete }: Props = $props();

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

  const statusVariant = $derived(
    ({ active: "success", archived: "muted", on_hold: "warning" } as Record<string, string>)[
      project.status
    ] ?? "default"
  ) as "success" | "muted" | "warning" | "default";

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

<ListCard title={project.name} href={`/projects/${project.id}`}>
  {#snippet media()}
    <div class="icon">
      <Briefcase size={20} />
    </div>
  {/snippet}

  {#snippet titleSuffix()}
    <Badge variant={statusVariant} size="sm">{statusLabel}</Badge>
    {#if project.categoryName}
      <Badge variant="muted" size="sm">{project.categoryName}</Badge>
    {/if}
  {/snippet}

  {#snippet actions({ trigger, align })}
    <DropdownMenu {trigger} {align} items={menuItems} showTrigger={false} />
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

<AlertDialog
  bind:open={confirmDeleteOpen}
  showTrigger={false}
  title="Delete Project"
  description={`Are you sure you want to delete "${project.name}"? All tasks within this project will also be deleted.`}
  confirmLabel="Delete"
  onConfirm={handleDelete}
/>

<style>
  .icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2.5rem;
    height: 2.5rem;
    border-radius: 0.5rem;
    background: var(--color-primary, #6366f1);
    color: white;
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
