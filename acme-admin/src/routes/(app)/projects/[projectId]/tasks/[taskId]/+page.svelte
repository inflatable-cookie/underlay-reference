<script lang="ts">
import {
  useAuthenticatedData,
  useToasts
} from "@decodelabs/underlay/runtime";
import {
  AlertDialog as PoodleAlertDialog,
  Callout as PoodleCallout,
  Card as PoodleCard,
  Code as PoodleCode,
  DetailRow as PoodleDetailRow
  } from "@poodle/svelte-primitives";
  import { DetailSection as PoodleDetailSection,
  PageHeader as PoodlePageHeader,
  PageLoading } from "@poodle/svelte-composites";
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";
  import { adminCommands,
  type Task,
  type Label,
  type Project } from "@api-client";
  import { auth } from "$lib/stores/auth";
    import { TimeAgo } from "@poodle/svelte-primitives";
  import { Button as PoodleButton, MetaBar as PoodleMetaBar, MetaItem as PoodleMetaItem, Pill as PoodlePill } from "@poodle/svelte-primitives";
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
  let showDeleteConfirm = $state(false);

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
  <PageLoading presentation="inline" message="Loading task..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if task && project}
  <div class="task-detail__header">
  <PoodlePageHeader
    section="Task"
    title={task.title}
    backHref={`/projects/${data.projectId}`}
    backLabel={`Back to ${project.name}`}
    bannerMessage={task.status === "completed"
      ? `This task was completed on ${formatDate(task.completedAt)}.`
      : task.status === "cancelled"
        ? "This task has been cancelled."
        : undefined}
    bannerTone={task.status === "completed" ? "success" : "warning"}
  >
    <svelte:fragment slot="actions">
      <PoodleButton type="button" variant="secondary" on:click={handleEdit}>
        <Pencil size={16} />
        Edit
      </PoodleButton>
      <PoodleButton type="button" variant="ghost" tone="danger" on:click={() => (showDeleteConfirm = true)}>
        Delete
      </PoodleButton>
    </svelte:fragment>
  </PoodlePageHeader>
  <PoodleMetaBar ariaLabel="Task metadata">
    <PoodleMetaItem label="ID">
      <PoodleCode inline source={task.id} showCopyButton />
    </PoodleMetaItem>
    <PoodlePill tone={getTaskStatusTone(task.status)} appearance="badge" size="lg">
      {statusLabel}
    </PoodlePill>
    <PoodlePill tone={getTaskPriorityTone(task.priority)} appearance="badge" size="lg">
      {priorityLabel}
    </PoodlePill>
  </PoodleMetaBar>
  </div>

  <PoodleAlertDialog
    open={showDeleteConfirm}
    title="Delete Task"
    description={`Are you sure you want to delete "${task.title}"?`}
    confirmLabel="Delete"
    tone="danger"
    onConfirm={handleDelete}
    onCancel={() => {
      showDeleteConfirm = false;
    }}
  />

  <PoodleCard>
    <div class="detail-card-grid">
      <PoodleDetailSection title="Details" columns={2} separated={false}>
        <PoodleDetailRow label="Priority">
          <svelte:fragment slot="value">
            <PoodlePill
              tone={getTaskPriorityTone(task.priority)}
              appearance="badge"
              size="sm"
            >
              {priorityLabel}
            </PoodlePill>
          </svelte:fragment>
        </PoodleDetailRow>
        <PoodleDetailRow label="Due Date">
          <svelte:fragment slot="value">
            <span class="due-date">
              <Calendar size={14} />
              {formatDate(task.dueDate)}
            </span>
          </svelte:fragment>
        </PoodleDetailRow>
        {#if labels.length > 0}
          <div class="detail-span-full">
            <PoodleDetailRow label="Labels">
              <svelte:fragment slot="value">
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
              </svelte:fragment>
            </PoodleDetailRow>
          </div>
        {/if}
        {#if task.description}
          <div class="detail-span-full">
            <PoodleDetailRow label="Description" value={task.description} />
          </div>
        {/if}
      </PoodleDetailSection>

      <PoodleDetailSection title="Metadata" columns={2} separated={false}>
        <PoodleDetailRow label="Project">
          <svelte:fragment slot="value">
            <a href={`/projects/${project.id}`}>{project.name}</a>
          </svelte:fragment>
        </PoodleDetailRow>
        <PoodleDetailRow label="Position" value={String(task.position)} />
        <PoodleDetailRow label="Created">
          <svelte:fragment slot="value">
            <TimeAgo datetime={task.createdAt} tooltipFormat="datetime" />
          </svelte:fragment>
        </PoodleDetailRow>
        <PoodleDetailRow label="Updated">
          <svelte:fragment slot="value">
            <TimeAgo datetime={task.updatedAt} tooltipFormat="datetime" />
          </svelte:fragment>
        </PoodleDetailRow>
      </PoodleDetailSection>
    </div>
  </PoodleCard>
{:else}
  <PoodleCallout tone="danger" message="Task not found" announceMode="polite" />
{/if}

<style>
  .task-detail__header {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .detail-card-grid {
    display: grid;
    gap: 1rem;
  }

  .detail-span-full {
    grid-column: 1 / -1;
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

  a {
    color: var(--link-color, #3b82f6);
    text-decoration: none;
  }

  a:hover {
    text-decoration: underline;
  }
</style>
