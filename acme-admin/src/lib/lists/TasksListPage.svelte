<script lang="ts">
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import { EntityListPage } from "@decodelabs/underlay/templates";
  import type { QueryParams } from "@decodelabs/underlay/client/query";
  import {
    Button,
    Field,
    ListCard,
    Pill,
    Select
  } from "@poodle/svelte";
  import { adminCommands, TaskStatus, type TaskWithLabels } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";
  import CheckSquare from "lucide-svelte/icons/check-square";

  interface Props {
    projectId: string;
    projectName?: string;
    projectArchived?: boolean;
    title?: string;
    subtitle?: string;
    eyebrow?: string;
    hideTitle?: boolean;
    headerLevel?: 1 | 2 | 3 | 4 | 5 | 6;
    query?: QueryParams;
    onQueryChange?: (query: QueryParams) => void;
  }

  let {
    projectId,
    projectName = "Project",
    projectArchived = false,
    title = "Tasks",
    subtitle,
    eyebrow,
    hideTitle = false,
    headerLevel = 2,
    query,
    onQueryChange
  }: Props = $props();

  const toastStore = useToasts();
  let localQuery = $state<QueryParams>({ page: 1, limit: 30 });
  let statusValue = $state<TaskStatus>(TaskStatus.Pending);

  const effectiveQuery = $derived(query ?? localQuery);

  function updateQuery(nextQuery: QueryParams) {
    if (onQueryChange) {
      onQueryChange(nextQuery);
      return;
    }

    localQuery = nextQuery;
  }

  async function dataLoader(fetch: typeof window.fetch, token: string | null, taskQuery: QueryParams) {
    if (!token) throw new Error("Not authenticated");
    return await adminCommands.listTasks(projectId, fetch, token, taskQuery);
  }

  function handleAddTask() {
    void gotoWithContext(`/projects/${projectId}/tasks/new`, {
      label: projectName,
      href: `/projects/${projectId}`,
      type: "detail"
    });
  }

  async function handleDeleteTask(taskId: string) {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    await adminCommands.softDeleteTask(projectId, taskId, fetch, token);
    toastStore.push({ variant: "success", message: "Task deleted" });
  }

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

  const isTaskCollectionFiltered = $derived(
    (effectiveQuery.filters?.length ?? 0) > 0 ||
      (effectiveQuery.sort ?? []).some((field) => field.field !== "position")
  );

  const taskBatchActions = $derived(
    projectArchived
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
              const result = await adminCommands.batchDeleteTasks(projectId, { ids }, fetch, token);
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
                projectId,
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
    projectArchived || isTaskCollectionFiltered
      ? undefined
      : {
          enabled: true,
          handler: async (orderedIds: string[]) => {
            const token = auth.getToken();
            if (!token) throw new Error("Not authenticated");
            await adminCommands.reorderTasks(projectId, { ids: orderedIds }, fetch, token);
          }
        }
  );
</script>

{#snippet statusDialog({ ids, onSubmit, onCancel }: { ids: string[]; onSubmit: (values: Record<string, unknown>) => void; onCancel: () => void })}
  <div class="tasks-list-page__status-dialog">
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
    <div class="tasks-list-page__dialog-actions">
      <Button type="button" variant="secondary" on:click={onCancel}>Cancel</Button>
      <Button type="button" variant="primary" on:click={() => onSubmit({ status: statusValue })}>
        Update {ids.length} task{ids.length === 1 ? "" : "s"}
      </Button>
    </div>
  </div>
{/snippet}

{#snippet taskCard(task: TaskWithLabels, ctx: { selectionMode: boolean; reorderMode: boolean; selected: boolean; onToggle: (selected: boolean) => void; refetch: () => Promise<void> })}
  <ListCard
    title={task.title}
    href={ctx.selectionMode || ctx.reorderMode ? undefined : `/projects/${projectId}/tasks/${task.id}`}
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

<EntityListPage
  {title}
  {subtitle}
  {eyebrow}
  {hideTitle}
  {headerLevel}
  {dataLoader}
  presentation="cards"
  renderItem={taskCard as never}
  filters={taskFilters}
  query={effectiveQuery}
  onQueryChange={updateQuery}
  batchActions={taskBatchActions as never}
  reorder={taskReorderConfig as never}
  onReorderError={async (error) => {
    const message = error instanceof Error ? error.message : String(error);
    toastStore.push({ variant: "info", message: `Reorder conflict: ${message}. Please refresh and try again.` });
    return message;
  }}
  onAdd={projectArchived ? undefined : handleAddTask}
  addLabel="Add Task"
/>

<style>
  .tasks-list-page__status-dialog {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .tasks-list-page__dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
  }
</style>
