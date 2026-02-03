<script lang="ts">
  import { ListCard, Badge, DropdownMenu, DropdownItem, ConfirmAction, ProgressBar } from "@decodelabs/underlay/components";
  import { gotoWithContext } from "@decodelabs/underlay/client";
  import type { ProjectWithCounts } from "@acme/client";
  import Briefcase from "lucide-svelte/icons/briefcase";
  import Pencil from "lucide-svelte/icons/pencil";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import MoreVertical from "lucide-svelte/icons/more-vertical";

  interface Props {
    project: ProjectWithCounts;
    onDelete?: (projectId: string) => void;
  }

  let { project, onDelete }: Props = $props();

  const progress = $derived(
    project.taskCount > 0
      ? Math.round((project.completedTaskCount / project.taskCount) * 100)
      : 0
  );

  const statusLabel = $derived({
    active: "Active",
    archived: "Archived",
    on_hold: "On Hold"
  }[project.status] ?? project.status);

  const statusVariant = $derived({
    active: "success",
    archived: "muted",
    on_hold: "warning"
  }[project.status] ?? "default");

  function handleEdit() {
    void gotoWithContext(`/projects/${project.id}/edit`, {
      label: "Projects",
      href: "/projects",
      type: "list"
    });
  }

  function handleDelete() {
    onDelete?.(project.id);
  }
</script>

<ListCard
  title={project.name}
  href={`/projects/${project.id}`}
  variant="standard"
>
  {#snippet media()}
    <div class="icon">
      <Briefcase size={20} />
    </div>
  {/snippet}

  {#snippet badges()}
    <Badge variant={statusVariant} size="sm">{statusLabel}</Badge>
    {#if project.categoryName}
      <Badge variant="subtle" size="sm">{project.categoryName}</Badge>
    {/if}
  {/snippet}

  {#snippet meta()}
    <div class="task-progress">
      <span class="progress-label">{project.completedTaskCount}/{project.taskCount} tasks</span>
      {#if project.taskCount > 0}
        <ProgressBar value={progress} max={100} size="sm" />
      {/if}
    </div>
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
          title="Delete Project"
          message={`Are you sure you want to delete "${project.name}"? All tasks within this project will also be deleted.`}
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
