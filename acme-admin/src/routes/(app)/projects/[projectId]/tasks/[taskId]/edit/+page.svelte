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
  import {
    adminCommands,
    type Label,
    type Project,
    type Task,
    TaskPriority,
    TaskStatus
  } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { isPreconditionFailed } from "$lib/utils/api-errors";
  import TaskForm from "$lib/forms/TaskForm.svelte";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  let title = $state("");
  let description = $state("");
  let notes = $state<NightfireDraftValue>({ schema: "acme:task/notes@1" });
  let status = $state<string>(TaskStatus.Pending);
  let priority = $state<string>(TaskPriority.Medium);
  let dueDate = $state("");
  let selectedLabelIds = $state<string[]>([]);
  let submitting = $state(false);
  let fieldErrors = $state<Record<string, string> | null>(null);
  let initialized = $state(false);
  let labelsInitialized = $state(false);
  let currentEtag = $state<string | null>(null);

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

  $effect(() => {
    if (pageData.data?.taskEtag) {
      currentEtag = pageData.data.taskEtag;
    }
  });

  $effect(() => {
    if (!task || initialized) return;
    title = task.title;
    description = task.description ?? "";
    notes = task.notes ?? { schema: "acme:task/notes@1" };
    status = task.status;
    priority = task.priority;
    dueDate = task.dueDate?.split("T")[0] ?? "";
    initialized = true;
  });

  const defaultBackHref = untrack(() => `/projects/${data.projectId}/tasks/${data.taskId}`);
  const { backInfo } = consumeNavigationContext("Back to task", defaultBackHref);
  const computedBackInfo = $derived(
    computeBackInfo(
      backInfo,
      task
        ? {
            href: `/projects/${data.projectId}/tasks/${task.id}`,
            label: `Back to ${task.title}`
          }
        : undefined
    )
  );

  let labels = $state<Label[]>([]);

  $effect(() => {
    if (!task) return;
    const token = auth.getToken();
    if (!token) return;
    Promise.all([
      adminCommands.listLabels(data.projectId, fetch, token),
      adminCommands.getTaskLabels(data.projectId, data.taskId, fetch, token)
    ]).then(([allLabels, taskLabels]) => {
      labels = allLabels;
      if (!labelsInitialized) {
        selectedLabelIds = taskLabels.map((label) => label.id);
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
      const result = await adminCommands.updateTaskWithEtag(
        data.projectId,
        data.taskId,
        {
          title: title.trim(),
          description: description.trim() || null,
          notes: prepareNightfireForSave(notes) ?? null,
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

      return {
        success: true,
        redirectTo: `/projects/${data.projectId}/tasks/${data.taskId}`
      };
    } catch (e) {
      if (isPreconditionFailed(e)) {
        const latest = await adminCommands.getTaskWithEtag(data.projectId, data.taskId, fetch, token);
        currentEtag = latest.etag;
        title = latest.data.title;
        description = latest.data.description ?? "";
        notes = latest.data.notes ?? { schema: "acme:task/notes@1" };
        status = latest.data.status;
        priority = latest.data.priority;
        dueDate = latest.data.dueDate?.split("T")[0] ?? "";
        const latestLabels = await adminCommands.getTaskLabels(data.projectId, data.taskId, fetch, token);
        selectedLabelIds = latestLabels.map((label) => label.id);

        return {
          success: false,
          error: "This task was changed in another session. Review the latest values, reapply your edits, and save again."
        };
      }

      return {
        success: false,
        error: e instanceof Error ? e.message : "Failed to update task"
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
  title="Edit task"
  subtitle={project ? `For project: ${project.name}` : undefined}
  backHref={computedBackInfo.href}
  backLabel={computedBackInfo.label}
  backIsContextual={computedBackInfo.isContextual ?? false}
  loading={pageData.loading}
  loadingMessage="Loading task..."
  error={pageData.error}
  {fieldErrors}
  onSubmit={handleSubmit}
  onResult={handleResult}
  navigate={goto}
>
  {#if task && project}
    <TaskForm
      mode="edit"
      bind:title
      bind:description
      bind:notes
      bind:status
      bind:priority
      bind:dueDate
      bind:selectedLabelIds
      {labels}
      {statusItems}
      {priorityItems}
      errors={fieldErrors}
      {submitting}
      cancelHref={computedBackInfo.href}
      onCancel={() => goto(computedBackInfo.href)}
    />
  {/if}
</EntityFormPage>
