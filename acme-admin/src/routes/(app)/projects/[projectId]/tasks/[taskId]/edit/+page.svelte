<script lang="ts">
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";
  import { adminCommands, type Task, type Project, type Label, TaskStatus, TaskPriority } from "acme-client";
  import { auth } from "$lib/stores/auth";
  import { isPreconditionFailed } from "$lib/utils/api-errors";
  import { useAuthenticatedData, PageHeader, useToasts } from "@decodelabs/underlay/patterns";
  import { Button, PageLoading, FormError, TextInput, TextArea, Select, Field } from "@decodelabs/underlay/components";

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
</script>

{#if pageData.loading}
  <PageLoading message="Loading task..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if task && project}
  <PageHeader
    section="Edit Task"
    backHref={`/projects/${data.projectId}/tasks/${data.taskId}`}
    backLabel={`Back to ${task.title}`}
  />

  <div class="form-container">
    <form onsubmit={handleSubmit}>
      {#if error}
        <FormError message={error} />
      {/if}

      <Field label="Title" required>
        <TextInput
          bind:value={title}
          placeholder="Enter task title"
          disabled={submitting}
        />
      </Field>

      <Field label="Description">
        <TextArea
          bind:value={description}
          placeholder="Enter task description (optional)"
          rows={4}
          disabled={submitting}
        />
      </Field>

      <div class="form-row">
        <Field label="Status">
          <Select
            value={status}
            onchange={(val) => { status = val; }}
            items={statusItems}
            disabled={submitting}
          />
        </Field>

        <Field label="Priority">
          <Select
            value={priority}
            onchange={(val) => { priority = val; }}
            items={priorityItems}
            disabled={submitting}
          />
        </Field>
      </div>

      <Field label="Due Date">
        <TextInput
          type="date"
          bind:value={dueDate}
          disabled={submitting}
        />
      </Field>

      {#if allLabels.length > 0}
        <Field label="Labels">
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
        </Field>
      {/if}

      <div class="form-actions">
        <Button type="button" variant="secondary" onclick={() => goto(`/projects/${data.projectId}/tasks/${data.taskId}`)} disabled={submitting}>
          Cancel
        </Button>
        <Button type="submit" variant="primary" disabled={submitting}>
          {submitting ? "Saving..." : "Save Changes"}
        </Button>
      </div>
    </form>
  </div>
{:else}
  <FormError message="Task not found" />
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

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    padding-top: 0.5rem;
  }
</style>
