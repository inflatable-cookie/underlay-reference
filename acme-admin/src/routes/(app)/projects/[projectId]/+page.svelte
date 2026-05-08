<script lang="ts">
  import { goto } from "$app/navigation";
  import {
    EntityDetailPage,
    EntityDetail,
    EntityAttributeList,
    EntityDetailModule
  } from "@decodelabs/underlay/templates";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import TasksListPage from "$lib/lists/TasksListPage.svelte";
  import {
    Code,
    Pill,
    Progress,
    TimeAgo
  } from "@poodle/svelte";
  import type { PageData } from "./$types";
  import {
    adminCommands,
    TaskStatus,
    type Project
  } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";
  import { getProjectStatusTone } from "$lib/utils/accents";

  let { data }: { data: PageData } = $props();
  const toastStore = useToasts();

  // Reactive project state — updated by dataLoader side effect
  let project = $state<Project | null>(null);
  let taskSummary = $state({ total: 0, completed: 0 });

  // Project data loader
  async function projectLoader(fetch: typeof window.fetch, token: string | null) {
    if (!token) throw new Error("Not authenticated");
    const [projectResult, totalTasksResult, completedTasksResult] = await Promise.all([
      adminCommands.getProject(data.projectId, fetch, token),
      adminCommands.listTasks(data.projectId, fetch, token, { page: 1, limit: 1 }),
      adminCommands.listTasks(data.projectId, fetch, token, {
        page: 1,
        limit: 1,
        filters: [
          {
            field: "status",
            value: TaskStatus.Completed
          }
        ]
      })
    ]);
    project = projectResult;
    taskSummary = {
      total: totalTasksResult.total,
      completed: completedTasksResult.total
    };
    return projectResult;
  }

  // Actions
  function handleEdit() {
    if (!project) return;
    void gotoWithContext(`/projects/${project.id}/edit`, {
      label: project.name,
      href: `/projects/${project.id}`,
      type: "detail"
    });
  }

  async function handleDelete() {
    if (!project) return;
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    await adminCommands.softDeleteProject(project.id, fetch, token);
    toastStore.push({ variant: "success", message: "Project deleted" });
    await goto("/projects");
  }

  // Derived values
  const statusLabel = $derived(
    project
      ? {
          active: "Active",
          archived: "Archived",
          on_hold: "On Hold"
        }[project.status] ?? project.status
      : ""
  );

  const bannerMessage = $derived(
    project?.status === "archived"
      ? "This project is archived and tasks cannot be modified."
      : project?.status === "on_hold"
        ? "This project is on hold."
        : undefined
  );

  const bannerTone = $derived<"warning" | "info" | undefined>(
    project?.status === "archived" ? "warning" : project?.status === "on_hold" ? "info" : undefined
  );

  // Linked local packages currently expose nominally different Snippet types.
  // Cast at the page boundary instead of loosening the shared template API.
  const headerMeta = $derived.by(() => [
    { label: "ID", value: idSnippet as never },
    { label: "", value: statusSnippet as never, separator: false }
  ]);

  const breadcrumbs = $derived.by(() =>
    project?.categoryId && project.categoryName?.trim()
      ? [
          {
            label: project.categoryName.trim(),
            href: `/categories/${project.categoryId}`
          },
          {
            label: project.name
          }
        ]
      : []
  );

  const detailPageTabs = $derived.by(() => [
    {
      id: "details",
      label: "Details",
      content: detailsTabSnippet as never
    },
    {
      id: "tasks",
      label: "Tasks",
      content: tasksTabSnippet as never
    }
  ]);
</script>

<EntityDetailPage
  title={project?.name ?? "Project"}
  section="Project"
  backHref="/projects"
  backLabel="Back to projects"
  bannerMessage={bannerMessage}
  bannerTone={bannerTone}
  dataLoader={projectLoader}
  breadcrumbs={breadcrumbs}
  meta={headerMeta as never}
  actions={[
    { label: "Edit", handler: handleEdit },
    {
      label: "Delete",
      tone: "danger",
      confirm: {
        title: "Delete project",
        description: "Are you sure you want to delete this project?",
        confirmLabel: "Delete project",
        cancelLabel: "Keep project"
      },
      handler: handleDelete
    }
  ]}
  tabs={detailPageTabs as never}
/>

{#snippet idSnippet()}
  {#if project}
    <Code
      inline
      inlineVariant="plain"
      typography="inline"
      source={project.id}
      showCopyButton
    />
  {/if}
{/snippet}

{#snippet statusSnippet()}
  {#if project}
    <Pill tone={getProjectStatusTone(project.status)} appearance="badge" size="sm" typography="inherit">
      {statusLabel}
    </Pill>
  {/if}
{/snippet}

{#snippet progressSnippet()}
  {#if project}
    <div class="progress-cell">
      <span>{taskSummary.completed}/{taskSummary.total} tasks</span>
      {#if taskSummary.total > 0}
        <Progress
          value={Math.round((taskSummary.completed / taskSummary.total) * 100)}
          max={100}
          ariaLabel="Project completion progress"
        />
      {/if}
    </div>
  {/if}
{/snippet}

{#snippet createdSnippet()}
  {#if project}
    <TimeAgo datetime={project.createdAt} tooltipFormat="datetime" />
  {/if}
{/snippet}

{#snippet updatedSnippet()}
  {#if project}
    <TimeAgo datetime={project.updatedAt} tooltipFormat="datetime" />
  {/if}
{/snippet}

{#snippet categorySnippet()}
  {project?.categoryName?.trim() || "Unassigned"}
{/snippet}

{#snippet tasksTabSnippet(_project: Project)}
  <TasksListPage
    projectId={data.projectId}
    projectName={project?.name ?? "Project"}
    projectArchived={project?.status === "archived"}
    title="Tasks"
    hideTitle
    subtitle={`Tasks in ${project?.name ?? "project"}`}
    headerLevel={3}
  />
{/snippet}

{#snippet detailsTabSnippet(loadedProject: Project)}
  <EntityDetail>
    {#snippet children()}
      <EntityAttributeList
        title={null}
        columns={3}
        items={[
          {
            label: "Progress",
            value: progressSnippet as never,
            span: "full",
            layout: "stacked",
            presentation: "surface"
          },
          {
            label: "Category",
            value: categorySnippet as never,
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
          }
        ]}
      />

      {#if loadedProject.description}
        <EntityDetailModule>
          {#snippet children()}
            <div class="project-detail-copy">
              {loadedProject.description}
            </div>
          {/snippet}
        </EntityDetailModule>
      {/if}
    {/snippet}
  </EntityDetail>
{/snippet}

<style>
  .progress-cell {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .project-detail-copy {
    max-width: 42rem;
    color: var(--poodle-color-text-primary);
    font-size: var(--poodle-typography-body-size);
    line-height: 1.7;
    white-space: pre-wrap;
  }
</style>
