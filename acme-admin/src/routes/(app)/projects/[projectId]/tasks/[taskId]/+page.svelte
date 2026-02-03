<script lang="ts">
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";
  import { adminCommands, type Task, type Label, type Project } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import { useAuthenticatedData, PageHeader, useToasts, Banner } from "@decodelabs/underlay/patterns";
  import { Button, PageLoading, FormError, ConfirmAction, Badge } from "@decodelabs/underlay/components";
  import { gotoWithContext } from "@decodelabs/underlay/client";
  import Pencil from "lucide-svelte/icons/pencil";
  import Trash2 from "lucide-svelte/icons/trash-2";
  import ArrowLeft from "lucide-svelte/icons/arrow-left";
  import Calendar from "lucide-svelte/icons/calendar";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  const toastStore = useToasts();

  // Fetch task, project, and labels data
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const [task, project, labels] = await Promise.all([
        adminCommands.getTask(data.projectId, data.taskId, fetch, token),
        adminCommands.getProject(data.projectId, fetch, token),
        adminCommands.getTaskLabels(data.projectId, data.taskId, fetch, token)
      ]);
      return { task, project, labels };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { task: null as Task | null, project: null as Project | null, labels: [] as Label[] }
    }
  );

  // Trigger fetch when auth is ready
  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  const task = $derived(pageData.data?.task);
  const project = $derived(pageData.data?.project);
  const labels = $derived(pageData.data?.labels ?? []);

  const statusLabel = $derived(task ? {
    pending: "Pending",
    in_progress: "In Progress",
    completed: "Completed",
    cancelled: "Cancelled"
  }[task.status] ?? task.status : "");

  const statusVariant = $derived(task ? {
    pending: "muted",
    in_progress: "info",
    completed: "success",
    cancelled: "danger"
  }[task.status] ?? "default" : "default");

  const priorityLabel = $derived(task ? {
    low: "Low",
    medium: "Medium",
    high: "High",
    urgent: "Urgent"
  }[task.priority] ?? task.priority : "");

  const priorityVariant = $derived(task ? {
    low: "subtle",
    medium: "default",
    high: "warning",
    urgent: "danger"
  }[task.priority] ?? "default" : "default");

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
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await adminCommands.softDeleteTask(data.projectId, task.id, fetch, token);
      toastStore.push({ variant: "success", message: "Task deleted" });
      await goto(`/projects/${data.projectId}`);
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete task";
      toastStore.push({ variant: "error", message });
    }
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
</script>

{#if pageData.loading}
  <PageLoading message="Loading task..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if task && project}
  <PageHeader
    title={task.title}
    backHref={`/projects/${data.projectId}`}
    backLabel={`Back to ${project.name}`}
  >
    {#snippet actions()}
      <Button type="button" variant="secondary" onclick={handleEdit}>
        <Pencil size={16} />
        Edit
      </Button>
      <ConfirmAction
        title="Delete Task"
        message={`Are you sure you want to delete "${task.title}"?`}
        confirmLabel="Delete"
        onConfirm={handleDelete}
      >
        {#snippet trigger(triggerFn)}
          <Button type="button" variant="danger" onclick={triggerFn}>
            <Trash2 size={16} />
            Delete
          </Button>
        {/snippet}
      </ConfirmAction>
    {/snippet}
  </PageHeader>

  {#if task.status === "completed"}
    <Banner variant="success">
      This task was completed on {formatDate(task.completedAt)}.
    </Banner>
  {:else if task.status === "cancelled"}
    <Banner variant="warning">
      This task has been cancelled.
    </Banner>
  {/if}

  <div class="detail-grid">
    <section class="detail-section">
      <h2>Details</h2>
      <dl class="detail-list">
        <div class="detail-item">
          <dt>Status</dt>
          <dd>
            <Badge variant={statusVariant}>{statusLabel}</Badge>
          </dd>
        </div>
        <div class="detail-item">
          <dt>Priority</dt>
          <dd>
            <Badge variant={priorityVariant}>{priorityLabel}</Badge>
          </dd>
        </div>
        <div class="detail-item">
          <dt>Due Date</dt>
          <dd class="due-date">
            <Calendar size={14} />
            {formatDate(task.dueDate)}
          </dd>
        </div>
        {#if labels.length > 0}
          <div class="detail-item full">
            <dt>Labels</dt>
            <dd class="labels">
              {#each labels as label}
                <Badge variant="subtle" style="--badge-color: {label.color}">{label.name}</Badge>
              {/each}
            </dd>
          </div>
        {/if}
        {#if task.description}
          <div class="detail-item full">
            <dt>Description</dt>
            <dd class="description">{task.description}</dd>
          </div>
        {/if}
      </dl>
    </section>

    <section class="detail-section">
      <h2>Metadata</h2>
      <dl class="detail-list">
        <div class="detail-item">
          <dt>ID</dt>
          <dd><code>{task.id}</code></dd>
        </div>
        <div class="detail-item">
          <dt>Project</dt>
          <dd><a href={`/projects/${project.id}`}>{project.name}</a></dd>
        </div>
        <div class="detail-item">
          <dt>Position</dt>
          <dd>{task.position}</dd>
        </div>
        <div class="detail-item">
          <dt>Created</dt>
          <dd>{new Date(task.createdAt).toLocaleString()}</dd>
        </div>
        <div class="detail-item">
          <dt>Updated</dt>
          <dd>{new Date(task.updatedAt).toLocaleString()}</dd>
        </div>
      </dl>
    </section>
  </div>
{:else}
  <FormError message="Task not found" />
{/if}

<style>
  .detail-grid {
    display: grid;
    gap: 2rem;
    margin-top: 1.5rem;
  }

  .detail-section {
    background: var(--bg-surface, #fff);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 0.5rem;
    padding: 1.5rem;
  }

  .detail-section h2 {
    margin: 0 0 1rem;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary, #111827);
  }

  .detail-list {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
    margin: 0;
  }

  .detail-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .detail-item.full {
    grid-column: span 2;
  }

  .detail-item dt {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-secondary, #6b7280);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .detail-item dd {
    margin: 0;
    font-size: 0.875rem;
    color: var(--text-primary, #111827);
  }

  .due-date {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .labels {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .description {
    white-space: pre-wrap;
    line-height: 1.6;
  }

  code {
    font-family: monospace;
    font-size: 0.8em;
    background: var(--bg-muted, #f3f4f6);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }

  a {
    color: var(--link-color, #3b82f6);
    text-decoration: none;
  }

  a:hover {
    text-decoration: underline;
  }
</style>
