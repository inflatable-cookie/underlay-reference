<script lang="ts">
  import { goto } from "$app/navigation";
  import { untrack } from "svelte";
  import type { PageData } from "./$types";
  import { EntityFormPage } from "@inflatable-cookie/underlay/templates";
  import type { SpaFormResult } from "@inflatable-cookie/underlay/patterns";
  import { useAuthenticatedData } from "@inflatable-cookie/underlay/runtime/auth";
  import { computeBackInfo, consumeNavigationContext } from "@inflatable-cookie/underlay/runtime/navigation";
  import type { NightfireDraftValue } from "@inflatable-cookie/underlay/nightfire/validation";
  import { prepareNightfireForSave } from "@inflatable-cookie/underlay/nightfire/validation";
  import { adminCommands, type Label, type Project, TaskPriority } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import TaskForm from "$lib/forms/TaskForm.svelte";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  let title = $state("");
  let description = $state("");
  let notes = $state<NightfireDraftValue>({ schema: "acme:task/notes@1", blocks: [] });
  let priority = $state<string>(TaskPriority.Medium);
  let dueDate = $state("");
  let selectedLabelIds = $state<string[]>([]);
  let submitting = $state(false);
  let fieldErrors = $state<Record<string, string> | null>(null);

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

  const defaultBackHref = untrack(() => `/projects/${data.projectId}`);
  const { backInfo } = consumeNavigationContext("Back to project", defaultBackHref);
  const computedBackInfo = $derived(
    computeBackInfo(
      backInfo,
      project
        ? {
            href: `/projects/${project.id}`,
            label: `Back to ${project.name}`
          }
        : undefined
    )
  );

  let labels = $state<Label[]>([]);

  $effect(() => {
    if (!project) return;
    const token = auth.getToken();
    if (!token) return;
    adminCommands.listLabels(data.projectId, fetch, token, { limit: 100 }).then((result) => {
      labels = result.data;
    });
  });

  const priorityItems = [
    { value: TaskPriority.Low, label: "Low" },
    { value: TaskPriority.Medium, label: "Medium" },
    { value: TaskPriority.High, label: "High" },
    { value: TaskPriority.Urgent, label: "Urgent" }
  ];

  async function handleSubmit(): Promise<SpaFormResult> {
    const token = auth.getToken();
    if (!token) {
      return { success: false, error: "Not authenticated" };
    }

    const errors: Record<string, string> = {};
    if (!title.trim()) errors.title = "Title is required";

    if (Object.keys(errors).length > 0) {
      return {
        success: false,
        error: "Please fill in all required fields",
        fieldErrors: errors
      };
    }

    submitting = true;

    try {
      const task = await adminCommands.createTask(
        data.projectId,
        {
          title: title.trim(),
          description: description.trim() || null,
          notes: prepareNightfireForSave(notes) ?? null,
          priority,
          dueDate: dueDate || null,
          labelIds: selectedLabelIds.length > 0 ? selectedLabelIds : undefined
        },
        fetch,
        token
      );

      return {
        success: true,
        redirectTo: `/projects/${data.projectId}/tasks/${task.id}`
      };
    } catch (e) {
      return {
        success: false,
        error: e instanceof Error ? e.message : "Failed to create task"
      };
    } finally {
      submitting = false;
    }
  }

  function handleResult(result: SpaFormResult) {
    fieldErrors = result.fieldErrors ?? null;
  }
</script>

<EntityFormPage
  section="Tasks"
  title="New task"
  subtitle={project ? `For project: ${project.name}` : undefined}
  backHref={computedBackInfo.href}
  backLabel={computedBackInfo.label}
  backIsContextual={computedBackInfo.isContextual ?? false}
  loading={pageData.loading}
  loadingMessage="Loading project..."
  error={pageData.error}
  {fieldErrors}
  onSubmit={handleSubmit}
  onResult={handleResult}
  navigate={goto}
>
  {#if project}
    <TaskForm
      mode="create"
      bind:title
      bind:description
      bind:notes
      bind:priority
      bind:dueDate
      bind:selectedLabelIds
      {labels}
      {priorityItems}
      errors={fieldErrors}
      {submitting}
      cancelHref={computedBackInfo.href}
      onCancel={() => goto(computedBackInfo.href)}
    />
  {/if}
</EntityFormPage>
