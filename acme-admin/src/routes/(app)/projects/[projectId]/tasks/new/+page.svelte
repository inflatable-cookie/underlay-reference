<script lang="ts">
  import { Callout as PoodleCallout } from "@poodle/svelte-primitives";
  import { PageHeader as PoodlePageHeader } from "@poodle/svelte-composites";
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";
  import { adminCommands, type Project, type Label, TaskPriority } from "acme-client";
  import { auth } from "$lib/stores/auth";
  import { useAuthenticatedData, useToasts } from "@decodelabs/underlay/patterns";
  import { PageLoading } from "@decodelabs/underlay/components";
  import {
    Button as PoodleButton,
    Field as PoodleField,
    FormActions as PoodleFormActions,
    Select as PoodleSelect,
    TextArea as PoodleTextArea,
    TextInput as PoodleTextInput
  } from "@poodle/svelte-primitives";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  const toastStore = useToasts();

  // Form state
  let title = $state("");
  let description = $state("");
  let priority = $state<string>(TaskPriority.Medium);
  let dueDate = $state("");
  let selectedLabelIds = $state<string[]>([]);
  let submitting = $state(false);
  let error = $state<string | null>(null);

  // Fetch project data
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const project = await adminCommands.getProject(data.projectId, fetch, token);
      return { project };
    },
    {
      defaultValue: { project: null as Project | null }
    }
  );

  const project = $derived(pageData.data?.project);

  // Lazy-load labels (non-blocking, fetched after page renders)
  let labels = $state<Label[]>([]);

  $effect(() => {
    if (!project) return;
    const token = auth.getToken();
    if (!token) return;
    adminCommands.listLabels(data.projectId, fetch, token).then((result) => {
      labels = result;
    });
  });

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
      const task = await adminCommands.createTask(
        data.projectId,
        {
          title: title.trim(),
          description: description.trim() || null,
          priority,
          dueDate: dueDate || null,
          labelIds: selectedLabelIds.length > 0 ? selectedLabelIds : undefined
        },
        fetch,
        token
      );

      toastStore.push({ variant: "success", message: "Task created" });
      await goto(`/projects/${data.projectId}/tasks/${task.id}`);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to create task";
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
  <PageLoading message="Loading..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if project}
  <PoodlePageHeader
    title="New Task"
    backHref={`/projects/${data.projectId}`}
    backLabel={`Back to ${project.name}`}
  />

  <div class="form-container">
    <form onsubmit={handleSubmit}>
      {#if error}
        <PoodleCallout tone="danger" message={error} announceMode="polite" />
      {/if}

      <PoodleField
        id="task-create-title"
        label="Title"
        error={titleError()}
        validationState={validationState(titleError())}
        required
        let:describedBy
        let:validationState={titleValidationState}
      >
        <PoodleTextInput
          id="task-create-title"
          value={title}
          describedBy={describedBy}
          validationState={titleValidationState}
          placeholder="Enter task title"
          disabled={submitting}
          on:valueChange={(event) => { title = event.detail.value; }}
        />
      </PoodleField>

      <PoodleField
        id="task-create-description"
        label="Description"
        let:describedBy
      >
        <PoodleTextArea
          id="task-create-description"
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
          id="task-create-priority"
          label="Priority"
          let:describedBy
        >
          <PoodleSelect
            id="task-create-priority"
            value={priority}
            describedBy={describedBy}
            options={priorityItems}
            disabled={submitting}
            on:valueChange={(event) => { priority = event.detail.value; }}
          />
        </PoodleField>

        <PoodleField
          id="task-create-due-date"
          label="Due Date"
          let:describedBy
        >
          <PoodleTextInput
            id="task-create-due-date"
            type="date"
            value={dueDate}
            describedBy={describedBy}
            disabled={submitting}
            on:valueChange={(event) => { dueDate = event.detail.value; }}
          />
        </PoodleField>
      </div>

      {#if labels.length > 0}
        <PoodleField id="task-create-labels" label="Labels">
          <div class="labels-grid">
            {#each labels as label}
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
        <PoodleButton type="button" variant="secondary" disabled={submitting} on:click={() => goto(`/projects/${data.projectId}`)}>
          Cancel
        </PoodleButton>
        <PoodleButton type="submit" variant="primary" disabled={submitting}>
          {submitting ? "Creating..." : "Create Task"}
        </PoodleButton>
      </PoodleFormActions>
    </form>
  </div>
{:else}
  <PoodleCallout tone="danger" message="Project not found" announceMode="polite" />
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
