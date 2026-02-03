<script lang="ts">
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";
  import { adminCommands, type Project, type Label, TaskPriority } from "@acme/client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import { useAuthenticatedData, PageHeader, useToasts } from "@decodelabs/underlay/patterns";
  import { Button, PageLoading, FormError, Input, Textarea, Select, FormField } from "@decodelabs/underlay/components";

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

  // Fetch project and labels data
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const [project, labels] = await Promise.all([
        adminCommands.getProject(data.projectId, fetch, token),
        adminCommands.listLabels(data.projectId, fetch, token)
      ]);
      return { project, labels };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { project: null as Project | null, labels: [] as Label[] }
    }
  );

  // Trigger fetch when auth is ready
  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  const project = $derived(pageData.data?.project);
  const labels = $derived(pageData.data?.labels ?? []);

  const priorityOptions = [
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
</script>

{#if pageData.loading}
  <PageLoading message="Loading..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if project}
  <PageHeader
    title="New Task"
    backHref={`/projects/${data.projectId}`}
    backLabel={`Back to ${project.name}`}
  />

  <div class="form-container">
    <form onsubmit={handleSubmit}>
      {#if error}
        <FormError message={error} />
      {/if}

      <FormField label="Title" required>
        <Input
          bind:value={title}
          placeholder="Enter task title"
          disabled={submitting}
        />
      </FormField>

      <FormField label="Description">
        <Textarea
          bind:value={description}
          placeholder="Enter task description (optional)"
          rows={4}
          disabled={submitting}
        />
      </FormField>

      <div class="form-row">
        <FormField label="Priority">
          <Select
            bind:value={priority}
            options={priorityOptions}
            disabled={submitting}
          />
        </FormField>

        <FormField label="Due Date">
          <Input
            type="date"
            bind:value={dueDate}
            disabled={submitting}
          />
        </FormField>
      </div>

      {#if labels.length > 0}
        <FormField label="Labels">
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
        </FormField>
      {/if}

      <div class="form-actions">
        <Button type="button" variant="secondary" onclick={() => goto(`/projects/${data.projectId}`)} disabled={submitting}>
          Cancel
        </Button>
        <Button type="submit" variant="primary" disabled={submitting}>
          {submitting ? "Creating..." : "Create Task"}
        </Button>
      </div>
    </form>
  </div>
{:else}
  <FormError message="Project not found" />
{/if}

<style>
  .form-container {
    max-width: 40rem;
    margin-top: 1.5rem;
    background: var(--bg-surface, #fff);
    border: 1px solid var(--border-color, #e5e7eb);
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
    border: 1px solid var(--border-color, #e5e7eb);
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
