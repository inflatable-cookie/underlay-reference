<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { EntityDetailPage, EntityList } from "@decodelabs/underlay/templates";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import { buildQueryString, parseQueryParams } from "@decodelabs/underlay/client/query";
  import type { QueryParams } from "@decodelabs/underlay/client/query";
  import {
    Code,
    ListCard,
    Pill,
    Progress,
    TimeAgo,
    Button,
    Field,
    Select
  } from "@poodle/svelte";
  import type { PageData } from "./$types";
  import {
    adminCommands,
    TaskStatus,
    type Project,
    type TaskWithLabels
  } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";
  import { getProjectStatusTone } from "$lib/utils/accents";
  import CheckSquare from "lucide-svelte/icons/check-square";

  let { data }: { data: PageData } = $props();
  const toastStore = useToasts();

  // Reactive project state — updated by dataLoader side effect
  let project = $state<Project | null>(null);
  let taskSummary = $state({ total: 0, completed: 0 });

  // URL sync for task filters
  const query = $derived(parseQueryParams($page.url.searchParams));

  function updateUrl(nextQuery: QueryParams) {
    const url = new URL($page.url);
    url.search = buildQueryString(nextQuery);
    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

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

  // Task data loader
  async function taskLoader(fetch: typeof window.fetch, token: string | null, taskQuery: QueryParams) {
    if (!token) throw new Error("Not authenticated");
    return await adminCommands.listTasks(data.projectId, fetch, token, taskQuery);
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

  function handleAddTask() {
    if (!project) return;
    void gotoWithContext(`/projects/${project.id}/tasks/new`, {
      label: project.name,
      href: `/projects/${project.id}`,
      type: "detail"
    });
  }

  async function handleDeleteTask(taskId: string) {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    await adminCommands.softDeleteTask(data.projectId, taskId, fetch, token);
    toastStore.push({ variant: "success", message: "Task deleted" });
  }

  // Batch status dialog state
  let statusValue = $state<TaskStatus>(TaskStatus.Pending);

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

  const isProjectArchived = $derived(project?.status === "archived");
  const isTaskCollectionFiltered = $derived(
    (query.filters?.length ?? 0) > 0 ||
      (query.sort ?? []).some((field) => field.field !== "position")
  );
  const taskBatchActions = $derived(
    isProjectArchived
      ? []
      : [
          {
            id: "delete",
            label: "Delete",
            tone: "danger" as const,
            icon: "trash-2",
            confirm: {
              title: "Delete tasks",
              description: (count: number) =>
                `Are you sure you want to delete ${count} task${count === 1 ? "" : "s"}?`,
              confirmLabel: "Delete tasks",
              cancelLabel: "Keep tasks"
            },
            handler: async (ids: string[]) => {
              const token = auth.getToken();
              if (!token) throw new Error("Not authenticated");
              const result = await adminCommands.batchDeleteTasks(data.projectId, { ids }, fetch, token);
              toastStore.push({
                variant: "success",
                message: `Deleted ${result.deleted} task${result.deleted === 1 ? "" : "s"}`
              });
            }
          },
          {
            id: "status",
            label: "Update Status",
            icon: "check-check",
            dialog: {
              title: "Update Task Status",
              content: statusDialog
            },
            handler: async (ids: string[], values?: Record<string, unknown>) => {
              const token = auth.getToken();
              if (!token) throw new Error("Not authenticated");
              const result = await adminCommands.batchUpdateTaskStatus(
                data.projectId,
                { ids, status: values?.status as TaskStatus },
                fetch,
                token
              );
              toastStore.push({
                variant: "success",
                message: `Updated ${result.updated} task${result.updated === 1 ? "" : "s"}`
              });
            }
          }
        ]
  );
  const taskReorderConfig = $derived(
    isProjectArchived || isTaskCollectionFiltered
      ? undefined
      : {
          enabled: true,
          handler: async (orderedIds: string[]) => {
            const token = auth.getToken();
            if (!token) throw new Error("Not authenticated");
            await adminCommands.reorderTasks(data.projectId, { ids: orderedIds }, fetch, token);
          }
        }
  );

  const taskFilters = [
    {
      id: "status",
      type: "select" as const,
      label: "Status",
      options: [
        { value: "All", label: "All statuses" },
        { value: TaskStatus.Pending, label: "Pending" },
        { value: TaskStatus.InProgress, label: "In Progress" },
        { value: TaskStatus.Completed, label: "Completed" }
      ]
    },
    {
      id: "priority",
      type: "select" as const,
      label: "Priority",
      options: [
        { value: "All", label: "All priorities" },
        { value: "low", label: "Low" },
        { value: "medium", label: "Medium" },
        { value: "high", label: "High" },
        { value: "urgent", label: "Urgent" }
      ]
    },
    {
      id: "sort",
      type: "sort" as const,
      label: "Sort",
      sortFields: [
        { key: "title", label: "Title" },
        { key: "status", label: "Status" },
        { key: "priority", label: "Priority" },
        { key: "dueDate", label: "Due Date", defaultDirection: "asc" as const },
        { key: "position", label: "Position" },
        { key: "createdAt", label: "Created", defaultDirection: "desc" as const }
      ]
    }
  ];

  // Linked local packages currently expose nominally different Snippet types.
  // Cast at the page boundary instead of loosening the shared template API.
  const detailPageMeta = $derived.by(() => [
    { label: "ID", value: idSnippet as never },
    { label: "Status", value: statusSnippet as never }
  ]);

  const detailPageSections = $derived.by(() => [
    {
      title: "Details",
      columns: 2 as const,
      separated: false,
      items: [
        { label: "Progress", value: progressSnippet as never },
        ...(project?.description ? [{ label: "Description", value: project.description }] : [])
      ]
    },
    {
      title: "Metadata",
      columns: 2 as const,
      separated: false,
      items: [
        { label: "Category", value: project?.categoryId ?? "None" },
        { label: "Created", value: createdSnippet as never },
        { label: "Updated", value: updatedSnippet as never }
      ]
    }
  ]);

  const detailPageTabs = $derived.by(() => [
    {
      id: "tasks",
      label: "Tasks",
      content: tasksTabSnippet as never
    }
  ]);
</script>

{#snippet statusDialog({ ids, onSubmit, onCancel }: { ids: string[]; onSubmit: (values: Record<string, unknown>) => void; onCancel: () => void })}
  <div class="status-dialog-content">
    <Field id="batch-status" label="New Status">
      <Select
        id="batch-status"
        value={statusValue}
        items={[
          { value: TaskStatus.Pending, label: "Pending" },
          { value: TaskStatus.InProgress, label: "In Progress" },
          { value: TaskStatus.Completed, label: "Completed" }
        ]}
        onchange={(value) => {
          statusValue = value as TaskStatus;
        }}
      />
    </Field>
    <div class="dialog-actions">
      <Button type="button" variant="secondary" on:click={onCancel}>Cancel</Button>
      <Button type="button" variant="primary" on:click={() => onSubmit({ status: statusValue })}>
        Update {ids.length} task{ids.length === 1 ? "" : "s"}
      </Button>
    </div>
  </div>
{/snippet}

{#snippet taskCard(task: TaskWithLabels, ctx: { selectionMode: boolean; reorderMode: boolean; selected: boolean; onToggle: (selected: boolean) => void })}
  <ListCard
    title={task.title}
    href={ctx.selectionMode || ctx.reorderMode ? undefined : `/projects/${data.projectId}/tasks/${task.id}`}
    layout="compact"
    selectable={ctx.selectionMode}
    selected={ctx.selected}
    on:selectedChange={(event) => ctx.onToggle(event.detail.selected)}
  >
    <svelte:fragment slot="leading">
      <CheckSquare size={16} />
    </svelte:fragment>
    <svelte:fragment slot="badges">
      <Pill
        tone={task.status === "completed" ? "success" : "neutral"}
        appearance="badge"
        size="sm"
      >
        {task.status === "completed" ? "Done" : task.status === "in_progress" ? "In Progress" : "Pending"}
      </Pill>
      <Pill
        tone={task.priority === "urgent" ? "danger" : "neutral"}
        appearance="badge"
        size="sm"
      >
        {task.priority}
      </Pill>
    </svelte:fragment>
  </ListCard>
{/snippet}

<EntityDetailPage
  title={project?.name ?? "Project"}
  section="Project"
  backHref="/projects"
  backLabel="Back to projects"
  bannerMessage={bannerMessage}
  bannerTone={bannerTone}
  dataLoader={projectLoader}
  meta={detailPageMeta as never}
  detailSections={detailPageSections as never}
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
    <Code inline source={project.id} showCopyButton />
  {/if}
{/snippet}

{#snippet statusSnippet()}
  {#if project}
    <Pill tone={getProjectStatusTone(project.status)} appearance="badge" size="lg">
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

{#snippet tasksTabSnippet()}
  <EntityList
    dataLoader={taskLoader}
    presentation="cards"
    renderItem={taskCard as never}
    filters={taskFilters}
    {query}
    onQueryChange={updateUrl}
    batchActions={taskBatchActions as never}
    reorder={taskReorderConfig as never}
    onReorderError={async (error) => {
      const message = error instanceof Error ? error.message : String(error);
      toastStore.push({ variant: "info", message: `Reorder conflict: ${message}. Please refresh and try again.` });
      return message;
    }}
    onAdd={isProjectArchived ? undefined : handleAddTask}
    addLabel="Add Task"
  />
{/snippet}

<style>
  .progress-cell {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .status-dialog-content {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
  }
</style>
