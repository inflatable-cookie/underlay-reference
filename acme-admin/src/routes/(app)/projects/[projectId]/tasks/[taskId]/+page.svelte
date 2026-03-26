<script lang="ts">
  import { Callout as PoodleCallout } from "@poodle/svelte-primitives";
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";
  import { adminCommands, type Task, type Label, type Project } from "acme-client";
  import { auth } from "$lib/stores/auth";
  import {
    useAuthenticatedData,
    PageHeader,
    DetailMeta,
    DetailMetaId,
    DetailMetaItem,
    DetailMetaSeparator,
    useToasts
  } from "@decodelabs/underlay/patterns";
  import { PageLoading, ConfirmAction, DetailsCard, DetailsSection, DetailsItem, TimeAgo } from "@decodelabs/underlay/components";
  import { Button as PoodleButton, Pill as PoodlePill } from "@poodle/svelte-primitives";
  import { gotoWithContext } from "@decodelabs/underlay/client";
  import { getTaskStatusTone, getTaskPriorityTone } from "$lib/utils/accents";
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
      defaultValue: { task: null as Task | null, project: null as Project | null, labels: [] as Label[] }
    }
  );

  const task = $derived(pageData.data?.task);
  const project = $derived(pageData.data?.project);
  const labels = $derived(pageData.data?.labels ?? []);

  const statusLabel = $derived(task ? {
    pending: "Pending",
    in_progress: "In Progress",
    completed: "Completed",
    cancelled: "Cancelled"
  }[task.status] ?? task.status : "");

  const priorityLabel = $derived(task ? {
    low: "Low",
    medium: "Medium",
    high: "High",
    urgent: "Urgent"
  }[task.priority] ?? task.priority : "");

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
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if task && project}
  <PageHeader
    section="Task"
    title={task.title}
    backHref={`/projects/${data.projectId}`}
    backLabel={`Back to ${project.name}`}
    bannerMessage={task.status === "completed"
      ? `This task was completed on ${formatDate(task.completedAt)}.`
      : task.status === "cancelled"
        ? "This task has been cancelled."
        : undefined}
    bannerVariant={task.status === "completed" ? "success" : "warning"}
  >
    <DetailMeta>
      <DetailMetaId value={task.id} />
      <DetailMetaSeparator />
      <DetailMetaItem>
        <PoodlePill tone={getTaskStatusTone(task.status)} appearance="badge" size="lg">
          {statusLabel}
        </PoodlePill>
      </DetailMetaItem>
      <DetailMetaItem>
        <PoodlePill tone={getTaskPriorityTone(task.priority)} appearance="badge" size="lg">
          {priorityLabel}
        </PoodlePill>
      </DetailMetaItem>
    </DetailMeta>

    {#snippet actions()}
      <PoodleButton type="button" variant="secondary" on:click={handleEdit}>
        <Pencil size={16} />
        Edit
      </PoodleButton>
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

  <DetailsCard>
    <DetailsSection legend="Details">
      <DetailsItem label="Priority">
        <PoodlePill
          tone={getTaskPriorityTone(task.priority)}
          appearance="badge"
          size="sm"
        >
          {priorityLabel}
        </PoodlePill>
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
              <PoodlePill
                tone="neutral"
                appearance="badge"
                size="sm"
              >
                {label.name}
              </PoodlePill>
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
  <PoodleCallout tone="danger" message="Task not found" announceMode="polite" />
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
