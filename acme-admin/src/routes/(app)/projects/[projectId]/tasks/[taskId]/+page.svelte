<script lang="ts">
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";
  import { adminCommands, type Task, type Label, type Project } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import {
    useAuthenticatedData,
    PageHeader,
    PageHeaderMeta,
    PageHeaderMetaRow,
    PageHeaderMetaItem,
    PageHeaderMetaSeparator,
    useToasts,
    Banner
  } from "@decodelabs/underlay/patterns";
  import { Button, Code, PageLoading, FormError, ConfirmAction, Badge, Pill, DetailsCard, DetailsSection, DetailsItem, TimeAgo } from "@decodelabs/underlay/components";
  import { gotoWithContext } from "@decodelabs/underlay/client";
  import Pencil from "lucide-svelte/icons/pencil";
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

  type BadgeVariant = "default" | "success" | "warning" | "danger" | "info" | "muted";
  const statusVariant = $derived<BadgeVariant>(task ? ({
    pending: "muted",
    in_progress: "info",
    completed: "success",
    cancelled: "danger"
  } as Record<string, BadgeVariant>)[task.status] ?? "default" : "default");

  const priorityLabel = $derived(task ? {
    low: "Low",
    medium: "Medium",
    high: "High",
    urgent: "Urgent"
  }[task.priority] ?? task.priority : "");

  const priorityVariant = $derived<BadgeVariant>(task ? ({
    low: "muted",
    medium: "default",
    high: "warning",
    urgent: "danger"
  } as Record<string, BadgeVariant>)[task.priority] ?? "default" : "default");

  function getStatusAccent(status: string): string {
    switch (status) {
      case "pending": return "#6b7280";
      case "in_progress": return "#3b82f6";
      case "completed": return "#10b981";
      case "cancelled": return "#ef4444";
      default: return "#64748b";
    }
  }

  function getPriorityAccent(priority: string): string {
    switch (priority) {
      case "low": return "#6b7280";
      case "medium": return "#64748b";
      case "high": return "#f59e0b";
      case "urgent": return "#ef4444";
      default: return "#64748b";
    }
  }

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
    section="Task"
    title={task.title}
    backHref={`/projects/${data.projectId}`}
    backLabel={`Back to ${project.name}`}
  >
    <PageHeaderMeta>
      <PageHeaderMetaRow>
        <PageHeaderMetaItem label="ID">
          <Code copy>{task.id}</Code>
        </PageHeaderMetaItem>
        <PageHeaderMetaSeparator />
        <Pill accent={getStatusAccent(task.status)}>{statusLabel}</Pill>
        <Pill accent={getPriorityAccent(task.priority)}>{priorityLabel}</Pill>
      </PageHeaderMetaRow>
    </PageHeaderMeta>

    {#snippet actions()}
      <Button type="button" variant="secondary" onclick={handleEdit}>
        <Pencil size={16} />
        Edit
      </Button>
      <ConfirmAction
        title="Delete Task"
        description={`Are you sure you want to delete "${task.title}"?`}
        confirmLabel="Delete"
        triggerLabel="Delete"
        triggerVariant="danger"
        onConfirm={handleDelete}
      />
    {/snippet}
  </PageHeader>

  {#if task.status === "completed"}
    <Banner variant="success" message={`This task was completed on ${formatDate(task.completedAt)}.`} />
  {:else if task.status === "cancelled"}
    <Banner variant="warning" message="This task has been cancelled." />
  {/if}

  <DetailsCard>
    <DetailsSection legend="Details">
      <DetailsItem label="Priority">
        <Badge variant={priorityVariant}>{priorityLabel}</Badge>
      </DetailsItem>
      <DetailsItem label="Due Date">
        <span class="due-date">
          <Calendar size={14} />
          {formatDate(task.dueDate)}
        </span>
      </DetailsItem>
      {#if labels.length > 0}
        <DetailsItem label="Labels" span="full">
          <div class="labels">
            {#each labels as label}
              <Badge variant="muted" style="--badge-color: {label.color}">{label.name}</Badge>
            {/each}
          </div>
        </DetailsItem>
      {/if}
      {#if task.description}
        <DetailsItem label="Description" value={task.description} span="full" />
      {/if}
    </DetailsSection>

    <DetailsSection legend="Metadata">
      <DetailsItem label="Project">
        <a href={`/projects/${project.id}`}>{project.name}</a>
      </DetailsItem>
      <DetailsItem label="Position" value={task.position} />
      <DetailsItem label="Created">
        <TimeAgo date={task.createdAt} tooltipFormat="datetime" />
      </DetailsItem>
      <DetailsItem label="Updated">
        <TimeAgo date={task.updatedAt} tooltipFormat="datetime" />
      </DetailsItem>
    </DetailsSection>
  </DetailsCard>
{:else}
  <FormError message="Task not found" />
{/if}

<style>
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

  a {
    color: var(--link-color, #3b82f6);
    text-decoration: none;
  }

  a:hover {
    text-decoration: underline;
  }
</style>
