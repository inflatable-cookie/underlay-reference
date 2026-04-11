<script lang="ts">
import {
  useToasts
} from "@decodelabs/underlay/runtime/feedback";
import {
  useAuthenticatedData
} from "@decodelabs/underlay/runtime/auth";
import {
  Callout as PoodleCallout,
  Card as PoodleCard } from "@poodle/svelte";
  import { PageHeader as PoodlePageHeader,
  PageLoading } from "@poodle/svelte";
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";
  import { adminCommands,
  type Task,
  type Project,
  type Label,
  TaskStatus,
  TaskPriority } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { isPreconditionFailed } from "$lib/utils/api-errors";
    import {
    Button as PoodleButton,
    Field as PoodleField,
    FormActions as PoodleFormActions,
    Select as PoodleSelect,
    TextInput as PoodleTextInput
  } from "@poodle/svelte";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  const toastStore = useToasts();

  // Form state
  let title = $state("");
  let description = $state("");
  let status = $state<string>(TaskStatus.Pending);
  let priority = $state<string>(TaskPriority.Medium);
  let dueDate = $state("");
  let selectedLabelIds = $state<string[]>([]);
  let submitting = $state(false);
  let error = $state<string | null>(null);
  let initialized = $state(false);

  // Fetch task and project data
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const [taskResult, project] = await Promise.all([
        adminCommands.getTaskWithEtag(data.projectId, data.taskId, fetch, token),
        adminCommands.getProject(data.projectId, fetch, token)
      ]);
      return { task: taskResult.data, taskEtag: taskResult.etag, project };
    },
    {
      defaultValue: {
        task: null as Task | null,
        taskEtag: null as string | null,
        project: null as Project | null
      }
    }
  );

  const task = $derived(pageData.data?.task);
  const project = $derived(pageData.data?.project);
  let currentEtag = $state<string | null>(null);

  $effect(() => {
    if (pageData.data?.taskEtag) {
      currentEtag = pageData.data.taskEtag;
    }
  });

  // Initialize form when task data loads
  $effect(() => {
    if (task && !initialized) {
      title = task.title;
      description = task.description ?? "";
      status = task.status;
      priority = task.priority;
      dueDate = task.dueDate?.split("T")[0] ?? "";
      initialized = true;
    }
  });

  // Lazy-load labels (non-blocking, fetched after page renders)
  let allLabels = $state<Label[]>([]);
  let labelsInitialized = $state(false);

  $effect(() => {
    if (!task) return;
    const token = auth.getToken();
    if (!token) return;
    Promise.all([
      adminCommands.listLabels(data.projectId, fetch, token),
      adminCommands.getTaskLabels(data.projectId, data.taskId, fetch, token)
    ]).then(([labels, taskLabels]) => {
      allLabels = labels;
      if (!labelsInitialized) {
        selectedLabelIds = taskLabels.map((l: Label) => l.id);
        labelsInitialized = true;
      }
    });
  });

  const statusItems = [
    { value: TaskStatus.Pending, label: "Pending" },
    { value: TaskStatus.InProgress, label: "In Progress" },
    { value: TaskStatus.Completed, label: "Completed" }
  ];

  const priorityItems = [
    { value: TaskPriority.Low, label: "Low" },
    { value: TaskPriority.Medium, label: "Medium" },
    { value: TaskPriority.High, label: "High" },
    { value: TaskPriority.Urgent, label: "Urgent" }
  ];

  async function handleSubmit(e: Event) {
    e.preventDefault();

    if (!title.trim()) {
      error = "Title is required";
      return;
    }

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    submitting = true;
    error = null;

    try {
      const result = await adminCommands.updateTaskWithEtag(
        data.projectId,
        data.taskId,
        {
          title: title.trim(),
          description: description.trim() || null,
          status,
          priority,
          dueDate: dueDate || null,
          labelIds: selectedLabelIds
        },
        fetch,
        token,
        { ifMatch: currentEtag ?? undefined }
      );
      currentEtag = result.etag;

      toastStore.push({ variant: "success", message: "Task updated" });
      await goto(`/projects/${data.projectId}/tasks/${data.taskId}`);
    } catch (e) {
      if (isPreconditionFailed(e)) {
        const latest = await adminCommands.getTaskWithEtag(data.projectId, data.taskId, fetch, token);
        currentEtag = latest.etag;
        title = latest.data.title;
        description = latest.data.description ?? "";
        status = latest.data.status;
        priority = latest.data.priority;
        dueDate = latest.data.dueDate?.split("T")[0] ?? "";
        const latestLabels = await adminCommands.getTaskLabels(data.projectId, data.taskId, fetch, token);
        selectedLabelIds = latestLabels.map((label) => label.id);
        error = "This task was changed in another session. Review the latest values, reapply your edits, and save again.";
        toastStore.push({ variant: "error", message: error });
        return;
      }
      error = e instanceof Error ? e.message : "Failed to update task";
      toastStore.push({ variant: "error", message: error });
    } finally {
      submitting = false;
    }
  }

  function toggleLabel(labelId: string) {
    if (selectedLabelIds.includes(labelId)) {
      selectedLabelIds = selectedLabelIds.filter(id => id !== labelId);
    } else {
      selectedLabelIds = [...selectedLabelIds, labelId];
    }
  }

  function validationState(error?: string | null) {
    return error ? "invalid" : "none";
  }

  function titleError(): string | null {
    return error === "Title is required" ? error : null;
  }
</script>

{#if pageData.loading}
  <PageLoading presentation="inline" message="Loading task..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if task && project}
  <PoodlePageHeader
    title="Edit Task"
    backHref={`/projects/${data.projectId}/tasks/${data.taskId}`}
    backLabel={`Back to ${task.title}`}
    subtitle={`For project: ${project.name}`}
  />

  <div class="form-container">
    <PoodleCard>
      <form onsubmit={handleSubmit}>
      {#if error}
        <PoodleCallout tone="danger" message={error} announceMode="polite" />
      {/if}

      <PoodleField
        id="task-edit-title"
        label="Title"
        error={titleError()}
        validationState={validationState(titleError())}
        required
        let:describedBy
        let:validationState={titleValidationState}
      >
        <PoodleTextInput
          id="task-edit-title"
          value={title}
          describedBy={describedBy}
          validationState={titleValidationState}
          placeholder="Enter task title"
          disabled={submitting}
          on:valueChange={(event) => { title = event.detail.value; }}
        />
      </PoodleField>

      <PoodleField
        id="task-edit-description"
        label="Description"
        let:describedBy
      >
        <PoodleTextInput
          id="task-edit-description"
          value={description}
          describedBy={describedBy}
          placeholder="Enter task description (optional)"
          rows={4}
          disabled={submitting}
          on:valueChange={(event) => { description = event.detail.value; }}
        />
      </PoodleField>

      <div class="form-row">
        <PoodleField
          id="task-edit-status"
          label="Status"
          let:describedBy
        >
          <PoodleSelect
            id="task-edit-status"
            value={status}
            describedBy={describedBy}
            options={statusItems}
            disabled={submitting}
            on:valueChange={(event) => { status = event.detail.value; }}
          />
        </PoodleField>

        <PoodleField
          id="task-edit-priority"
          label="Priority"
          let:describedBy
        >
          <PoodleSelect
            id="task-edit-priority"
            value={priority}
            describedBy={describedBy}
            options={priorityItems}
            disabled={submitting}
            on:valueChange={(event) => { priority = event.detail.value; }}
          />
        </PoodleField>
      </div>

      <PoodleField
        id="task-edit-due-date"
        label="Due Date"
        let:describedBy
      >
        <PoodleTextInput
          id="task-edit-due-date"
          type="date"
          value={dueDate}
          describedBy={describedBy}
          disabled={submitting}
          on:valueChange={(event) => { dueDate = event.detail.value; }}
        />
      </PoodleField>

      {#if allLabels.length > 0}
        <PoodleField id="task-edit-labels" label="Labels">
          <div class="labels-grid">
            {#each allLabels as label}
              <button
                type="button"
                class="label-chip"
                class:selected={selectedLabelIds.includes(label.id)}
                style="--label-color: {label.color}"
                onclick={() => toggleLabel(label.id)}
                disabled={submitting}
              >
                <span class="label-dot"></span>
                {label.name}
              </button>
            {/each}
          </div>
        </PoodleField>
      {/if}

      <PoodleFormActions align="end">
        <PoodleButton type="button" variant="secondary" disabled={submitting} on:click={() => goto(`/projects/${data.projectId}/tasks/${data.taskId}`)}>
          Cancel
        </PoodleButton>
        <PoodleButton type="submit" variant="primary" disabled={submitting}>
          {submitting ? "Saving..." : "Save Changes"}
        </PoodleButton>
      </PoodleFormActions>
      </form>
    </PoodleCard>
  </div>
{:else}
  <PoodleCallout tone="danger" message="Task not found" announceMode="polite" />
{/if}

<style>
  .form-container {
    max-width: 40rem;
    margin-top: 1.5rem;
    background: var(--underlay-color-surface, #fff);
    border: 1px solid var(--underlay-color-border-subtle, #e5e7eb);
    border-radius: 0.5rem;
    padding: 1.5rem;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .labels-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .label-chip {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.75rem;
    font-size: 0.875rem;
    background: var(--bg-muted, #f3f4f6);
    border: 1px solid var(--underlay-color-border-subtle, #e5e7eb);
    border-radius: 9999px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .label-chip:hover {
    background: var(--bg-hover, #e5e7eb);
  }

  .label-chip.selected {
    background: color-mix(in srgb, var(--label-color) 15%, white);
    border-color: var(--label-color);
  }

  .label-dot {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    background: var(--label-color);
  }
</style>
