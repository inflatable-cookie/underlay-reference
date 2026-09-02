<script lang="ts">
  import "@acme/ui/render";

  import { goto } from "$app/navigation";
  import { useToasts } from "@inflatable-cookie/underlay/runtime/feedback";
  import { useAuthenticatedData } from "@inflatable-cookie/underlay/runtime/auth";
  import { NightfireRenderer } from "@inflatable-cookie/underlay/nightfire/renderer";
  import type { NightfireValue as RenderableNightfireValue } from "@inflatable-cookie/underlay/nightfire/validation";
  import {
    AlertDialog,
    Button,
    Callout,
    Field,
    FormActions,
    FormDialog,
    PageHeader as PoodlePageHeader,
    PageLoading,
    Pill,
    Select,
    TextInput
  } from "@inflatable-cookie/poodle-svelte";
  import Plus from "lucide-svelte/icons/plus";
  import CheckSquare from "lucide-svelte/icons/check-square";
  import Trash2 from "lucide-svelte/icons/trash-2";

  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import * as userCommands from "@api-client/commands/user-commands.js";
  import type { UserProject, UserTask } from "@api-client/commands/user-commands.js";
  import type { PageData } from "./$types";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  const toastStore = useToasts();

  // Fetch project and tasks
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const [project, tasks] = await Promise.all([
        userCommands.getProject(data.projectId, fetch, token),
        userCommands.listTasks(data.projectId, fetch, token)
      ]);
      return { project, tasks };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { project: null as UserProject | null, tasks: [] as UserTask[] }
    }
  );

  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  const project = $derived(pageData.data?.project);
  const tasks = $derived(pageData.data?.tasks ?? []);

  // Task creation dialog
  let showCreateDialog = $state(false);
  let showDeleteConfirm = $state(false);
  let newTaskTitle = $state("");
  let newTaskDescription = $state("");
  let newTaskPriority = $state("medium");
  let creating = $state(false);

  const priorityOptions = [
    { value: "low", label: "Low" },
    { value: "medium", label: "Medium" },
    { value: "high", label: "High" },
    { value: "urgent", label: "Urgent" }
  ];

  function openCreateDialog() {
    newTaskTitle = "";
    newTaskDescription = "";
    newTaskPriority = "medium";
    showCreateDialog = true;
  }

  async function handleCreateTask() {
    if (!newTaskTitle.trim() || creating) return;

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    creating = true;

    try {
      await userCommands.createTask(
        data.projectId,
        {
          title: newTaskTitle.trim(),
          description: newTaskDescription.trim() || null,
          priority: newTaskPriority
        },
        fetch,
        token
      );
      toastStore.push({ variant: "success", message: "Task created" });
      showCreateDialog = false;
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to create task";
      toastStore.push({ variant: "error", message });
    } finally {
      creating = false;
    }
  }

  async function handleToggleTask(task: UserTask) {
    const token = auth.getToken();
    if (!token) return;

    const newStatus = task.status === "completed" ? "pending" : "completed";

    try {
      await userCommands.updateTask(
        data.projectId,
        task.id,
        { status: newStatus },
        fetch,
        token
      );
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to update task";
      toastStore.push({ variant: "error", message });
    }
  }

  async function handleDeleteTask(taskId: string) {
    const token = auth.getToken();
    if (!token) return;

    try {
      await userCommands.deleteTask(data.projectId, taskId, fetch, token);
      toastStore.push({ variant: "success", message: "Task deleted" });
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete task";
      toastStore.push({ variant: "error", message });
    }
  }

  async function handleDeleteProject() {
    const token = auth.getToken();
    if (!token) return;

    try {
      await userCommands.deleteProject(data.projectId, fetch, token);
      toastStore.push({ variant: "success", message: "Project deleted" });
      await goto("/dashboard");
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete project";
      toastStore.push({ variant: "error", message });
    }
  }

  const completedCount = $derived(tasks.filter(t => t.status === "completed").length);
</script>

{#if pageData.loading}
  <PageLoading presentation="inline" message="Loading project..." />
{:else if pageData.error}
  <Callout tone="danger" message={pageData.error} announceMode="assertive" />
{:else if project}
  <PoodlePageHeader title={project.name} backHref="/dashboard" backLabel="Back to projects">
    {#snippet actions()}
      <Button type="button" variant="ghost" tone="danger" onClick={() => (showDeleteConfirm = true)}>
        Delete
      </Button>
    {/snippet}
  </PoodlePageHeader>

  <AlertDialog
    open={showDeleteConfirm}
    title="Delete Project"
    description={`Are you sure you want to delete "${project.name}"? All tasks will be deleted.`}
    confirmLabel="Delete"
    tone="danger"
    onConfirm={handleDeleteProject}
    onCancel={() => {
      showDeleteConfirm = false;
    }}
  />

  {#if project.description}
    <div class="description">
      <NightfireRenderer value={project.description as RenderableNightfireValue} />
    </div>
  {/if}

  <div class="tasks-section">
    <div class="tasks-header">
      <div class="tasks-title">
        <h2>Tasks</h2>
        <span class="task-count">{completedCount}/{tasks.length} completed</span>
      </div>
      <Button type="button" variant="primary" size="sm" onClick={openCreateDialog}>
        <Plus size={14} />
        Add Task
      </Button>
    </div>

    {#if tasks.length === 0}
      <p class="empty-tasks">No tasks yet. Add your first task to get started.</p>
    {:else}
      <ul class="task-list">
        {#each tasks as task}
          <li class="task-item" class:completed={task.status === "completed"}>
            <button
              class="task-checkbox"
              onclick={() => handleToggleTask(task)}
              type="button"
              aria-label={task.status === "completed" ? `Mark \"${task.title}\" as pending` : `Mark \"${task.title}\" as completed`}
            >
              {#if task.status === "completed"}
                <CheckSquare size={20} />
              {:else}
                <div class="checkbox-empty"></div>
              {/if}
            </button>
            <div class="task-content">
              <span class="task-title">{task.title}</span>
              {#if task.description}
                <span class="task-description">{task.description}</span>
              {/if}
            </div>
            <div class="task-meta">
              <Pill
                tone={task.priority === "urgent" ? "danger" : "neutral"}
                appearance="badge"
                size="sm"
              >
                {task.priority}
              </Pill>
              <button
                class="delete-btn"
                onclick={() => handleDeleteTask(task.id)}
                type="button"
                aria-label={`Delete task \"${task.title}\"`}
              >
                <Trash2 size={14} />
              </button>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
{:else}
  <Callout tone="danger" message="Project not found" announceMode="assertive" />
{/if}

<FormDialog
  bind:open={showCreateDialog}
  title="Add Task"
  subtitle="Add a new task to this project."
  submitting={creating}
  showDefaultActions={false}
  onCancel={() => { showCreateDialog = false; }}
>
  <form
      id="create-task-form"
      onsubmit={(event) => {
        event.preventDefault();
        void handleCreateTask();
      }}
    >
      <div class="dialog-fields">
        <Field id="front-task-title" label="Task Title" required>
          {#snippet control({ describedBy })}
            <TextInput
              id="front-task-title"
              value={newTaskTitle}
              describedBy={describedBy}
              placeholder="What needs to be done?"
              disabled={creating}
              onValueChange={(nextValue) => { newTaskTitle = nextValue; }}
            />
          {/snippet}
        </Field>
        <Field id="front-task-description" label="Description">
          {#snippet control({ describedBy })}
            <TextInput
              id="front-task-description"
              value={newTaskDescription}
              describedBy={describedBy}
              placeholder="Optional details"
              rows={3}
              disabled={creating}
              onValueChange={(nextValue) => { newTaskDescription = nextValue; }}
            />
          {/snippet}
        </Field>
        <Field id="front-task-priority" label="Priority">
          {#snippet control({ describedBy })}
            <Select
              id="front-task-priority"
              value={newTaskPriority}
              describedBy={describedBy}
              options={priorityOptions}
              disabled={creating}
              onValueChange={(value) => { newTaskPriority = value; }}
            />
          {/snippet}
        </Field>
      </div>

  </form>
  {#snippet actions(submitting)}
    <FormActions align="end" showTopBorder>
      <Button type="button" variant="ghost" disabled={creating} onClick={() => (showCreateDialog = false)}>
        Cancel
      </Button>
      <Button type="submit" form="create-task-form" variant="primary" disabled={creating || !newTaskTitle.trim()}>
        {creating ? "Adding..." : "Add Task"}
      </Button>
    </FormActions>
  {/snippet}
</FormDialog>

<style>
  .description {
    margin: 0 0 1.5rem;
    color: var(--poodle-color-text-secondary);
  }

  .tasks-section {
    background: color-mix(in srgb, var(--poodle-color-background-panel) 96%, transparent);
    border: 1px solid color-mix(in srgb, var(--poodle-color-border-subtle) 82%, transparent);
    border-radius: 0.5rem;
    padding: 1.5rem;
  }

  .tasks-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .tasks-title {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
  }

  .tasks-title h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .task-count {
    font-size: 0.875rem;
    color: var(--poodle-color-text-secondary);
  }

  .empty-tasks {
    text-align: center;
    padding: 2rem;
    color: var(--text-secondary, #6b7280);
  }

  .task-list {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .task-item {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--bg-muted, #f9fafb);
    border-radius: 0.375rem;
    transition: background 0.15s;
  }

  .task-item:hover {
    background: var(--bg-hover, #f3f4f6);
  }

  .task-item.completed {
    opacity: 0.6;
  }

  .task-item.completed .task-title {
    text-decoration: line-through;
  }

  .task-checkbox {
    flex-shrink: 0;
    padding: 0;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--primary-color, #2563eb);
  }

  .checkbox-empty {
    width: 20px;
    height: 20px;
    border: 2px solid var(--border-color, #d1d5db);
    border-radius: 0.25rem;
  }

  .task-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .task-title {
    font-weight: 500;
  }

  .task-description {
    font-size: 0.875rem;
    color: var(--text-secondary, #6b7280);
  }

  .task-meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .delete-btn {
    padding: 0.25rem;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-secondary, #9ca3af);
    opacity: 0;
    transition: opacity 0.15s, color 0.15s;
  }

  .task-item:hover .delete-btn {
    opacity: 1;
  }

  .task-item:focus-within .delete-btn {
    opacity: 1;
  }

  .delete-btn:hover {
    color: var(--danger-color, #ef4444);
  }

  .delete-btn:focus-visible {
    opacity: 1;
    outline: 2px solid var(--primary-color, #2563eb);
    outline-offset: 2px;
    border-radius: 0.25rem;
  }

  .dialog-fields {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
</style>
