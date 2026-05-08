<script lang="ts">
  import "@acme/ui/render";
  import {
    EntityAttributeList,
    EntityDetail,
    EntityDetailModule,
    EntityDetailPage
  } from "@decodelabs/underlay/templates";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import { NightfireRenderer } from "@decodelabs/underlay/nightfire/renderer";
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";
  import { adminCommands, type Label, type Project, type Task } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { Code, Pill, TimeAgo } from "@poodle/svelte";
  import { getTaskPriorityTone, getTaskStatusTone } from "$lib/utils/accents";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  let task = $state<Task | null>(null);
  let project = $state<Project | null>(null);
  let labels = $state<Label[]>([]);

  async function taskLoader(fetch: typeof window.fetch, token: string | null) {
    if (!token) throw new Error("Not authenticated");

    const [nextTask, nextProject, nextLabels] = await Promise.all([
      adminCommands.getTask(data.projectId, data.taskId, fetch, token),
      adminCommands.getProject(data.projectId, fetch, token),
      adminCommands.getTaskLabels(data.projectId, data.taskId, fetch, token)
    ]);

    task = nextTask;
    project = nextProject;
    labels = nextLabels;
    return nextTask;
  }

  const statusLabel = $derived(
    task
      ? {
          pending: "Pending",
          in_progress: "In Progress",
          completed: "Completed",
          cancelled: "Cancelled"
        }[task.status] ?? task.status
      : ""
  );

  const priorityLabel = $derived(
    task
      ? {
          low: "Low",
          medium: "Medium",
          high: "High",
          urgent: "Urgent"
        }[task.priority] ?? task.priority
      : ""
  );

  const bannerMessage = $derived(
    task?.status === "completed"
      ? `This task was completed on ${formatDate(task.completedAt)}.`
      : task?.status === "cancelled"
        ? "This task has been cancelled."
        : undefined
  );

  const bannerTone = $derived<"success" | "warning">(
    task?.status === "completed" ? "success" : "warning"
  );

  function handleEdit() {
    if (!task || !project) return;
    void gotoWithContext(`/projects/${data.projectId}/tasks/${task.id}/edit`, {
      label: task.title,
      href: `/projects/${data.projectId}/tasks/${task.id}`,
      type: "detail"
    });
  }

  async function handleDelete() {
    if (!task) return;
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    await adminCommands.softDeleteTask(data.projectId, task.id, fetch, token);
    await goto(`/projects/${data.projectId}`);
  }

  function formatDate(dateString: string | null | undefined) {
    if (!dateString) return "Not set";
    return new Date(dateString).toLocaleDateString(undefined, {
      weekday: "long",
      year: "numeric",
      month: "long",
      day: "numeric"
    });
  }

  const headerMeta = $derived.by(() => [
    { label: "ID", value: idSnippet as never },
    { label: "", value: statusSnippet as never, separator: false },
    { label: "", value: prioritySnippet as never, separator: false }
  ]);

  const breadcrumbs = $derived.by(() => {
    if (!project) return [];

    const items: Array<{ label: string; href?: string }> = [];

    if (project.categoryId && project.categoryName?.trim()) {
      items.push({
        label: project.categoryName.trim(),
        href: `/categories/${project.categoryId}`
      });
    }

    items.push({
      label: project.name,
      href: `/projects/${project.id}`
    });

    return items;
  });

  const detailTabs = $derived.by(() => [
    {
      id: "details",
      label: "Details",
      content: detailsTabSnippet as never
    }
  ]);
</script>

<EntityDetailPage
  title={task?.title ?? "Task"}
  section="Task"
  showSubtitleWithBreadcrumbs
  breadcrumbsMarkLastCurrent={false}
  backHref={`/projects/${data.projectId}`}
  backLabel={project ? `Back to ${project.name}` : "Back to project"}
  bannerMessage={bannerMessage}
  bannerTone={bannerMessage ? bannerTone : undefined}
  dataLoader={taskLoader}
  breadcrumbs={breadcrumbs}
  meta={headerMeta as never}
  tabs={detailTabs as never}
  actions={[
    { label: "Edit", handler: handleEdit },
    {
      label: "Delete",
      tone: "danger",
      confirm: {
        title: "Delete task",
        description: task ? `Are you sure you want to delete "${task.title}"?` : "Are you sure you want to delete this task?",
        confirmLabel: "Delete task",
        cancelLabel: "Keep task"
      },
      handler: handleDelete
    }
  ]}
/>

{#snippet idSnippet()}
  {#if task}
    <Code
      inline
      inlineVariant="plain"
      typography="inline"
      source={task.id}
      showCopyButton
    />
  {/if}
{/snippet}

{#snippet statusSnippet()}
  {#if task}
    <Pill tone={getTaskStatusTone(task.status)} appearance="badge" size="sm" typography="inherit">
      {statusLabel}
    </Pill>
  {/if}
{/snippet}

{#snippet prioritySnippet()}
  {#if task}
    <Pill tone={getTaskPriorityTone(task.priority)} appearance="badge" size="sm" typography="inherit">
      {priorityLabel}
    </Pill>
  {/if}
{/snippet}

{#snippet dueDateSnippet()}
  {#if task?.dueDate}
    <TimeAgo datetime={task.dueDate} tooltipFormat="date" />
  {:else}
    Not set
  {/if}
{/snippet}

{#snippet projectSnippet()}
  {#if project}
    <a href={`/projects/${project.id}`} class="task-detail__project-link">{project.name}</a>
  {/if}
{/snippet}

{#snippet createdSnippet()}
  {#if task}
    <TimeAgo datetime={task.createdAt} tooltipFormat="datetime" />
  {/if}
{/snippet}

{#snippet updatedSnippet()}
  {#if task}
    <TimeAgo datetime={task.updatedAt} tooltipFormat="datetime" />
  {/if}
{/snippet}

{#snippet labelsSnippet()}
  {#if labels.length > 0}
    <div class="task-detail__labels">
      {#each labels as label}
        <Pill tone="neutral" appearance="badge" size="sm">
          {label.name}
        </Pill>
      {/each}
    </div>
  {:else}
    None
  {/if}
{/snippet}

{#snippet detailsTabSnippet(_loadedTask: Task)}
  <EntityDetail>
    {#snippet children()}
      <EntityAttributeList
        title={null}
        columns={2}
        items={[
          {
            label: "Due Date",
            value: dueDateSnippet as never,
            layout: "stacked",
            presentation: "surface"
          },
          {
            label: "Project",
            value: projectSnippet as never,
            layout: "stacked",
            presentation: "surface"
          },
          {
            label: "Created",
            value: createdSnippet as never,
            layout: "stacked",
            presentation: "surface"
          },
          {
            label: "Updated",
            value: updatedSnippet as never,
            layout: "stacked",
            presentation: "surface"
          },
          {
            label: "Labels",
            value: labelsSnippet as never,
            layout: "stacked",
            presentation: "surface",
            span: "full"
          }
        ]}
      />

      {#if task?.description}
        <EntityDetailModule>
          {#snippet children()}
            <div class="task-detail__copy">
              {_loadedTask.description}
            </div>
          {/snippet}
        </EntityDetailModule>
      {/if}

      {#if task?.notes}
        <EntityDetailModule>
          {#snippet children()}
            <div class="task-detail__notes">
              <NightfireRenderer value={_loadedTask.notes} />
            </div>
          {/snippet}
        </EntityDetailModule>
      {/if}
    {/snippet}
  </EntityDetail>
{/snippet}

<style>
  .task-detail__labels {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .task-detail__copy {
    max-width: 42rem;
    color: var(--poodle-color-text-primary);
    font-size: var(--poodle-typography-body-size);
    line-height: 1.7;
    white-space: pre-wrap;
  }

  .task-detail__notes {
    max-width: 42rem;
  }

  .task-detail__project-link {
    color: var(--poodle-color-accent-base);
    text-decoration: none;
  }

  .task-detail__project-link:hover {
    color: color-mix(in srgb, white 12%, var(--poodle-color-accent-base));
    text-decoration: underline;
  }

  .task-detail__project-link:focus-visible {
    outline: var(--poodle-border-width-focus) solid var(--poodle-color-accent-focusRing);
    outline-offset: 0.125rem;
    border-radius: var(--poodle-radius-control);
  }
</style>
