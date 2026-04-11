<script lang="ts">
import {
  createReorderController
} from "@decodelabs/underlay/runtime/data";
import {
  useToasts,
} from "@decodelabs/underlay/runtime/feedback";
import {
  useAuthenticatedData,
} from "@decodelabs/underlay/runtime/auth";
import {
  AlertDialog as PoodleAlertDialog,
  BulkActionBar as PoodleBulkActionBar,
  Callout as PoodleCallout,
  Card as PoodleCard,
  DetailItem as PoodleDetailItem,
  Dialog as PoodleDialog,
  Grid as PoodleGrid,
  ListCard as PoodleListCard,
  MetaBar as PoodleMetaBar,
  MetaItem as PoodleMetaItem,
  OrderBy as PoodleOrderBy,
  type BulkAction,
  type OrderByValue
  } from "@poodle/svelte";
  import { DetailSection as PoodleDetailSection,
  FilterToolbar,
  PageHeader as PoodlePageHeader,
  PageLoading,
  EditableList as PoodleReorderableList } from "@poodle/svelte";
  import type { PageData } from "./$types";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { adminCommands,
  type Project,
  type TaskWithLabels,
  TaskStatus,
  TaskPriority } from "@api-client";
  import { auth } from "$lib/stores/auth";
    import { Progress } from "@poodle/svelte";
  import { TimeAgo } from "@poodle/svelte";
  import {
    Button as PoodleButton,
    Code as PoodleCode,
    Field as PoodleField,
    Pill as PoodlePill,
    Select as PoodleSelect
  } from "@poodle/svelte";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import { parseQueryParams } from "@decodelabs/underlay/client/query";
  import { recoverReorderConflict } from "$lib/lists/reorder-conflicts";
  import { getProjectStatusTone } from "$lib/utils/accents";
  import Pencil from "lucide-svelte/icons/pencil";
  import Plus from "lucide-svelte/icons/plus";
  import CheckSquare from "lucide-svelte/icons/check-square";
  import ArrowUpDown from "lucide-svelte/icons/arrow-up-down";
  import CheckSquare2 from "lucide-svelte/icons/square-check";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  const toastStore = useToasts();
  let showDeleteConfirm = $state(false);
  let showBatchDeleteConfirm = $state(false);
  let showBatchStatusDialog = $state(false);
  let pendingBatchStatus = $state<TaskStatus>(TaskStatus.Pending);

  // Track URL for refetching when filters change
  let previousUrl = $state<string | null>(null);

  // Fetch project and tasks data
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const query = parseQueryParams($page.url.searchParams);
      const [project, tasks] = await Promise.all([
        adminCommands.getProject(data.projectId, fetch, token),
        adminCommands.listTasks(data.projectId, fetch, token, query)
      ]);
      return { project, tasks };
    },
    {
      defaultValue: { project: null as Project | null, tasks: [] as TaskWithLabels[] },
      onSuccess: () => {
        previousUrl = $page.url.search;
      }
    }
  );

  // Refetch when URL changes (for sorting/filtering)
  $effect(() => {
    const currentUrl = $page.url.search;
    if (previousUrl !== null && previousUrl !== currentUrl) {
      previousUrl = currentUrl;
      pageData.refetch();
    }
  });

  // Parse current state from URL
  const currentQuery = $derived(parseQueryParams($page.url.searchParams));

  // Convert URL sort to OrderByValue format
  const orderBy: OrderByValue = $derived(
    (currentQuery.sort ?? []).map((s) => ({ key: s.field, direction: s.direction }))
  );

  // Get filter values from URL
  const selectedStatus = $derived(
    currentQuery.filters?.find((f) => f.field === "status")?.value ?? "All"
  );
  const selectedPriority = $derived(
    currentQuery.filters?.find((f) => f.field === "priority")?.value ?? "All"
  );

  const sortFields = [
    { key: "title", label: "Title" },
    { key: "status", label: "Status" },
    { key: "priority", label: "Priority" },
    { key: "dueDate", label: "Due Date", defaultDirection: "asc" as const },
    { key: "position", label: "Position" },
    { key: "createdAt", label: "Created", defaultDirection: "desc" as const }
  ];

  const statusItems = [
    { value: "All", label: "All statuses" },
    { value: TaskStatus.Pending, label: "Pending" },
    { value: TaskStatus.InProgress, label: "In Progress" },
    { value: TaskStatus.Completed, label: "Completed" }
  ];

  const priorityItems = [
    { value: "All", label: "All priorities" },
    { value: TaskPriority.Low, label: "Low" },
    { value: TaskPriority.Medium, label: "Medium" },
    { value: TaskPriority.High, label: "High" },
    { value: TaskPriority.Urgent, label: "Urgent" }
  ];

  // Update URL when sort changes
  function handleSortChange(newOrderBy: OrderByValue) {
    const url = new URL($page.url);

    if (newOrderBy.length > 0) {
      const sortString = newOrderBy.map((s) => `${s.key}:${s.direction}`).join(",");
      url.searchParams.set("sort", sortString);
    } else {
      url.searchParams.delete("sort");
    }

    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  // Update URL when status filter changes
  function handleStatusChange(value: string) {
    const url = new URL($page.url);

    if (value && value !== "All") {
      url.searchParams.set("filter[status]", value);
    } else {
      url.searchParams.delete("filter[status]");
    }

    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  // Update URL when priority filter changes
  function handlePriorityChange(value: string) {
    const url = new URL($page.url);

    if (value && value !== "All") {
      url.searchParams.set("filter[priority]", value);
    } else {
      url.searchParams.delete("filter[priority]");
    }

    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  const project = $derived(pageData.data?.project);
  const tasks = $derived(pageData.data?.tasks ?? []);

  let isTaskReorderMode = $state(false);
  let taskReorderSubmitError = $state<string | null>(null);
  let isTaskSelectionMode = $state(false);
  let selectedTaskIds = $state<Set<string>>(new Set());
  let batchLoading = $state(false);

  // Clear selection when exiting selection mode
  $effect(() => {
    if (!isTaskSelectionMode) {
      selectedTaskIds = new Set();
    }
  });

  function toggleTaskSelectionMode() {
    isTaskSelectionMode = !isTaskSelectionMode;
    if (!isTaskSelectionMode) {
      selectedTaskIds = new Set();
    }
  }

  function handleTaskSelectionChange(taskId: string, selected: boolean) {
    const newSet = new Set(selectedTaskIds);
    if (selected) {
      newSet.add(taskId);
    } else {
      newSet.delete(taskId);
    }
    selectedTaskIds = newSet;
  }

  function clearTaskSelection() {
    selectedTaskIds = new Set();
    isTaskSelectionMode = false;
  }

  function selectAllTasks() {
    const allIds = tasks.map(t => t.id);
    if (selectedTaskIds.size === allIds.length) {
      clearTaskSelection();
      return;
    }
    selectedTaskIds = new Set(allIds);
  }

  async function handleBatchDeleteTasks() {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    batchLoading = true;
    showBatchDeleteConfirm = false;
    try {
      const result = await adminCommands.batchDeleteTasks(
        data.projectId,
        { ids: Array.from(selectedTaskIds) },
        fetch,
        token
      );
      toastStore.push({
        variant: "success",
        message: `Deleted ${result.deleted} task${result.deleted === 1 ? "" : "s"}`
      });
      selectedTaskIds = new Set();
      isTaskSelectionMode = false;
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete tasks";
      toastStore.push({ variant: "error", message });
    } finally {
      batchLoading = false;
    }
  }

  async function handleBatchUpdateTaskStatus(status: string) {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    batchLoading = true;
    showBatchStatusDialog = false;
    try {
      const result = await adminCommands.batchUpdateTaskStatus(
        data.projectId,
        { ids: Array.from(selectedTaskIds), status: status as TaskStatus },
        fetch,
        token
      );
      toastStore.push({
        variant: "success",
        message: `Updated ${result.updated} task${result.updated === 1 ? "" : "s"}`
      });
      selectedTaskIds = new Set();
      isTaskSelectionMode = false;
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to update tasks";
      toastStore.push({ variant: "error", message });
    } finally {
      batchLoading = false;
    }
  }

  const taskStatusOptions = [
    { value: TaskStatus.Pending, label: "Pending" },
    { value: TaskStatus.InProgress, label: "In Progress" },
    { value: TaskStatus.Completed, label: "Completed" }
  ];

  const batchActions: BulkAction[] = [
    { id: "status", label: "Update status", icon: "check-check" },
    { id: "delete", label: "Delete", icon: "trash-2", tone: "danger" }
  ];

  function handleBatchAction(actionId: string) {
    if (actionId === "delete") {
      showBatchDeleteConfirm = true;
      return;
    }

    if (actionId === "status") {
      pendingBatchStatus = TaskStatus.Pending;
      showBatchStatusDialog = true;
    }
  }

  // Map tasks to have 'id' field for reorder controller
  const reorderItems = $derived(
    tasks.map((t) => ({ ...t, id: t.id }))
  );

  // Create reorder controller for tasks
  const reorderController = $derived(
    createReorderController(reorderItems, async (orderedIds) => {
      const token = auth.getToken();
      if (!token) {
        toastStore.push({ variant: "error", message: "Not authenticated" });
        return;
      }
      await adminCommands.reorderTasks(data.projectId, { ids: orderedIds }, fetch, token);
    })
  );

  function enterTaskReorderMode() {
    taskReorderSubmitError = null;
    isTaskReorderMode = true;
  }

  async function handleTaskReorderSuccess() {
    taskReorderSubmitError = null;
    isTaskReorderMode = false;
    await pageData.refetch();
  }

  async function handleTaskReorderError(error: unknown): Promise<void | string> {
    await pageData.refetch();
    const latestItems = (pageData.data?.tasks ?? []).map((task) => ({ ...task, id: task.id }));
    const recovery = recoverReorderConflict({
      controller: reorderController,
      error,
      latestItems,
      entityLabel: "task"
    });

    if (!recovery.handled) return;

    toastStore.push({
      variant: "info",
      message: recovery.message
    });
    return recovery.message;
  }

  function exitTaskReorderMode() {
    reorderController.reset();
    taskReorderSubmitError = null;
    isTaskReorderMode = false;
  }

  function handleTaskReorderItems(items: typeof reorderController.pending) {
    reorderController.updatePending(items);
  }

  async function handleTaskReorderSubmit() {
    taskReorderSubmitError = null;
    try {
      await reorderController.submit();
      await handleTaskReorderSuccess();
    } catch (error) {
      const transformed = await handleTaskReorderError(error);
      taskReorderSubmitError = transformed ?? (error instanceof Error ? error.message : String(error));
    }
  }

  const completedTasks = $derived(tasks.filter(t => t.status === "completed").length);
  const progress = $derived(tasks.length > 0 ? Math.round((completedTasks / tasks.length) * 100) : 0);

  const statusLabel = $derived(project ? {
    active: "Active",
    archived: "Archived",
    on_hold: "On Hold"
  }[project.status] ?? project.status : "");

  function handleEdit() {
    if (!project) return;
    void gotoWithContext(`/projects/${project.id}/edit`, {
      label: project.name,
      href: `/projects/${project.id}`,
      type: "detail"
    });
  }

  async function handleDelete() {
    if (!project) return;

    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await adminCommands.softDeleteProject(project.id, fetch, token);
      toastStore.push({ variant: "success", message: "Project deleted" });
      await goto("/projects");
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete project";
      toastStore.push({ variant: "error", message });
    }
  }

  function handleAddTask() {
    if (!project) return;
    void gotoWithContext(`/projects/${project.id}/tasks/new`, {
      label: project.name,
      href: `/projects/${project.id}`,
      type: "detail"
    });
  }

  async function handleDeleteTask(taskId: string) {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    try {
      await adminCommands.softDeleteTask(data.projectId, taskId, fetch, token);
      toastStore.push({ variant: "success", message: "Task deleted" });
      await pageData.refetch();
    } catch (e) {
      const message = e instanceof Error ? e.message : "Failed to delete task";
      toastStore.push({ variant: "error", message });
    }
  }
</script>

{#if pageData.loading}
  <PageLoading presentation="inline" message="Loading project..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if project}
  <div class="project-detail__header">
  <PoodlePageHeader
    section="Project"
    title={project.name}
    backHref="/projects"
    backLabel="Back to projects"
    bannerMessage={project.status === "archived"
      ? "This project is archived and tasks cannot be modified."
      : project.status === "on_hold"
        ? "This project is on hold."
        : undefined}
    bannerTone={project.status === "archived" ? "warning" : "info"}
  >
    {#snippet actions()}
      <PoodleButton type="button" variant="secondary" on:click={handleEdit}>
        <Pencil size={16} />
        Edit
      </PoodleButton>
      <PoodleButton type="button" variant="ghost" tone="danger" on:click={() => (showDeleteConfirm = true)}>
        Delete
      </PoodleButton>
    {/snippet}
  </PoodlePageHeader>
  <PoodleMetaBar ariaLabel="Project metadata">
    <PoodleMetaItem label="ID">
      <PoodleCode inline source={project.id} showCopyButton />
    </PoodleMetaItem>
    <PoodlePill tone={getProjectStatusTone(project.status)} appearance="badge" size="lg">
      {statusLabel}
    </PoodlePill>
  </PoodleMetaBar>
  </div>

  <PoodleAlertDialog
    open={showDeleteConfirm}
    title="Delete Project"
    description={`Are you sure you want to delete "${project.name}"? All tasks within this project will also be deleted.`}
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
        <PoodleDetailItem presentation="surface" label="Progress">
          <svelte:fragment slot="value">
            <div class="progress-cell">
              <span>{completedTasks}/{tasks.length} tasks</span>
              {#if tasks.length > 0}
                <Progress value={progress} max={100} ariaLabel="Project completion progress" />
              {/if}
            </div>
          </svelte:fragment>
        </PoodleDetailItem>
        {#if project.description}
          <div class="detail-span-full">
            <PoodleDetailItem presentation="surface" label="Description" value={project.description} />
          </div>
        {/if}
      </PoodleDetailSection>

      <PoodleDetailSection title="Metadata" columns={2} separated={false}>
        <PoodleDetailItem presentation="surface" label="Category" value={project.categoryId ? project.categoryId : "None"} />
        <PoodleDetailItem presentation="surface" label="Created">
          <svelte:fragment slot="value">
            <TimeAgo datetime={project.createdAt} tooltipFormat="datetime" />
          </svelte:fragment>
        </PoodleDetailItem>
        <PoodleDetailItem presentation="surface" label="Updated">
          <svelte:fragment slot="value">
            <TimeAgo datetime={project.updatedAt} tooltipFormat="datetime" />
          </svelte:fragment>
        </PoodleDetailItem>
      </PoodleDetailSection>
    </div>
  </PoodleCard>

  <section class="tasks-section">
    <div class="tasks-header">
      <div class="tasks-heading">
        <h2>Tasks</h2>
        <PoodlePill tone="neutral" appearance="badge" size="sm">
          {tasks.length}
        </PoodlePill>
      </div>
      <div class="tasks-header-actions">
        {#if tasks.length > 1 && !isTaskSelectionMode}
          <PoodleButton
            type="button"
            variant="ghost"
            tone={isTaskReorderMode ? "danger" : "default"}
            size="sm"
            on:click={() => isTaskReorderMode ? exitTaskReorderMode() : enterTaskReorderMode()}
          >
            <ArrowUpDown size={14} />
            Reorder
          </PoodleButton>
        {/if}
        {#if tasks.length > 0 && !isTaskReorderMode}
          <PoodleButton
            type="button"
            variant="ghost"
            tone={isTaskSelectionMode ? "danger" : "default"}
            size="sm"
            on:click={toggleTaskSelectionMode}
          >
            <CheckSquare2 size={14} />
            {isTaskSelectionMode ? "Cancel" : "Select"}
          </PoodleButton>
        {/if}
        {#if !isTaskSelectionMode && !isTaskReorderMode}
          <PoodleButton type="button" variant="primary" size="sm" on:click={handleAddTask}>
            <Plus size={14} />
            Add Task
          </PoodleButton>
        {/if}
      </div>
    </div>

    {#if !isTaskReorderMode}
      <FilterToolbar ariaLabel="Task filters" summaryText="Filter tasks">
        <svelte:fragment slot="actions">
          <PoodleButton type="button" variant="ghost" size="sm" onclick={() => pageData.refetch()}>
            Refresh
          </PoodleButton>
        </svelte:fragment>
        <PoodleField id="project-tasks-filter-status" label="Status" let:describedBy>
          <PoodleSelect
            id="project-tasks-filter-status"
            value={selectedStatus}
            describedBy={describedBy}
            options={statusItems}
            placeholder="All statuses"
            on:valueChange={(event) => handleStatusChange(event.detail.value)}
          />
        </PoodleField>
        <PoodleField id="project-tasks-filter-priority" label="Priority" let:describedBy>
          <PoodleSelect
            id="project-tasks-filter-priority"
            value={selectedPriority}
            describedBy={describedBy}
            options={priorityItems}
            placeholder="All priorities"
            on:valueChange={(event) => handlePriorityChange(event.detail.value)}
          />
        </PoodleField>
        <PoodleField id="project-tasks-filter-sort" label="Sort">
          <PoodleOrderBy fields={sortFields} value={orderBy} onChange={handleSortChange} compact />
        </PoodleField>
      </FilterToolbar>
    {/if}

    {#if tasks.length === 0}
      <p class="empty-state">No tasks yet. Add your first task to get started.</p>
    {:else if isTaskReorderMode}
      <PoodleReorderableList
        items={reorderController.pending}
        ariaLabel="Reorder tasks"
        dirty={reorderController.isDirty}
        submitting={reorderController.isPending}
        errorMessage={taskReorderSubmitError}
        onsubmit={handleTaskReorderSubmit}
        oncancel={exitTaskReorderMode}
        on:reorder={(event) => handleTaskReorderItems(event.detail.items)}
      >
        {#snippet item(task)}
          <PoodleListCard
            title={task.title}
            layout="compact"
            showReorderHandle
          >
            <svelte:fragment slot="leading">
              <CheckSquare size={16} />
            </svelte:fragment>
            <svelte:fragment slot="badges">
              <PoodlePill
                tone={task.status === "completed" ? "success" : "neutral"}
                appearance="badge"
                size="sm"
              >
                {task.status === "completed" ? "Done" : task.status === "in_progress" ? "In Progress" : "Pending"}
              </PoodlePill>
            </svelte:fragment>
          </PoodleListCard>
        {/snippet}
      </PoodleReorderableList>
    {:else}
      <PoodleGrid columns="repeat(auto-fit, minmax(min(24em, 100%), 1fr))" gap="lg">
        {#each tasks as task}
          <PoodleListCard
            title={task.title}
            href={isTaskSelectionMode ? undefined : `/projects/${project.id}/tasks/${task.id}`}
            layout="compact"
            selectable={isTaskSelectionMode}
            selected={selectedTaskIds.has(task.id)}
            on:selectedChange={(event) => handleTaskSelectionChange(task.id, event.detail.selected)}
          >
            <svelte:fragment slot="leading">
              <CheckSquare size={16} />
            </svelte:fragment>
            <svelte:fragment slot="badges">
              <PoodlePill
                tone={task.status === "completed" ? "success" : "neutral"}
                appearance="badge"
                size="sm"
              >
                {task.status === "completed" ? "Done" : task.status === "in_progress" ? "In Progress" : "Pending"}
              </PoodlePill>
              <PoodlePill
                tone={task.priority === "urgent" ? "danger" : "neutral"}
                appearance="badge"
                size="sm"
              >
                {task.priority}
              </PoodlePill>
            </svelte:fragment>
          </PoodleListCard>
        {/each}
      </PoodleGrid>
    {/if}

    <PoodleBulkActionBar
      selectionCount={selectedTaskIds.size}
      totalCount={tasks.length}
      actions={batchActions}
      loading={batchLoading}
      showSelectAll
      allSelected={selectedTaskIds.size > 0 && selectedTaskIds.size === tasks.length}
      on:clear={clearTaskSelection}
      on:selectAll={selectAllTasks}
      on:action={(event) => handleBatchAction(event.detail.id)}
    />
  </section>
{:else}
  <PoodleCallout tone="danger" message="Project not found" announceMode="polite" />
{/if}

<PoodleAlertDialog
  open={showBatchDeleteConfirm}
  title="Delete selected tasks"
  description={`Are you sure you want to delete ${selectedTaskIds.size} selected task${selectedTaskIds.size === 1 ? "" : "s"}?`}
  confirmLabel={`Delete ${selectedTaskIds.size} task${selectedTaskIds.size === 1 ? "" : "s"}`}
  tone="danger"
  onConfirm={handleBatchDeleteTasks}
  onCancel={() => {
    showBatchDeleteConfirm = false;
  }}
/>

<PoodleDialog
  open={showBatchStatusDialog}
  title="Update task status"
  description={`Choose a new status for ${selectedTaskIds.size} selected task${selectedTaskIds.size === 1 ? "" : "s"}.`}
  on:openChange={(event) => {
    showBatchStatusDialog = event.detail.open;
  }}
>
  <PoodleField id="batch-status" label="Status">
    <PoodleSelect
      value={pendingBatchStatus}
      items={taskStatusOptions}
      on:valueChange={(event) => {
        pendingBatchStatus = event.detail.value as TaskStatus;
      }}
    />
  </PoodleField>

  <svelte:fragment slot="actions">
    <PoodleButton type="button" variant="ghost" on:click={() => {
      showBatchStatusDialog = false;
    }}>
      Cancel
    </PoodleButton>
    <PoodleButton
      type="button"
      variant="primary"
      disabled={!pendingBatchStatus}
      on:click={() => handleBatchUpdateTaskStatus(pendingBatchStatus)}
    >
      Update status
    </PoodleButton>
  </svelte:fragment>
</PoodleDialog>

<style>
  .project-detail__header {
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

  .progress-cell {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .tasks-section {
    margin-top: 2rem;
    background: var(--underlay-color-surface, #fff);
    border: 1px solid var(--underlay-color-border-subtle, #e5e7eb);
    border-radius: 0.5rem;
    padding: 1.5rem;
  }

  .tasks-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .tasks-heading {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .tasks-header h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary, #111827);
  }

  .tasks-header-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .empty-state {
    padding: 2rem;
    text-align: center;
    color: var(--text-secondary, #6b7280);
  }

</style>
