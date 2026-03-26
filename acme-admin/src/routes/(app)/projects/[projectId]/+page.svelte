<script lang="ts">
  import { Callout as PoodleCallout } from "@poodle/svelte-primitives";
  import type { PageData } from "./$types";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { adminCommands, type Project, type TaskWithLabels, TaskStatus, TaskPriority } from "acme-client";
  import { auth } from "$lib/stores/auth";
  import {
    useAuthenticatedData,
    PageHeader,
    DetailMeta,
    DetailMetaId,
    DetailMetaItem,
    DetailMetaSeparator,
    useToasts,
    ReorderableList,
    createReorderController,
    FilterBar
  } from "@decodelabs/underlay/patterns";
  import { PageLoading, ConfirmAction, ListGrid, ListCard, ProgressBar, OrderBy, DetailsCard, DetailsSection, DetailsItem, TimeAgo, type OrderByValue } from "@decodelabs/underlay/components";
  import {
    Button as PoodleButton,
    Field as PoodleField,
    Pill as PoodlePill,
    Select as PoodleSelect
  } from "@poodle/svelte-primitives";
  import { gotoWithContext, parseQueryParams } from "@decodelabs/underlay/client";
  import { BatchActionBar } from "$lib/components";
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
    selectedTaskIds = new Set(allIds);
  }

  async function handleBatchDeleteTasks() {
    const token = auth.getToken();
    if (!token) {
      toastStore.push({ variant: "error", message: "Not authenticated" });
      return;
    }

    batchLoading = true;
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
    isTaskReorderMode = true;
  }

  async function handleTaskReorderSuccess() {
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
    isTaskReorderMode = false;
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
  <PageLoading message="Loading project..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if project}
  <PageHeader
    section="Project"
    title={project.name}
    backHref="/projects"
    backLabel="Back to projects"
    bannerMessage={project.status === "archived"
      ? "This project is archived and tasks cannot be modified."
      : project.status === "on_hold"
        ? "This project is on hold."
        : undefined}
    bannerVariant={project.status === "archived" ? "warning" : "info"}
  >
    <DetailMeta>
      <DetailMetaId value={project.id} />
      <DetailMetaSeparator />
      <DetailMetaItem>
        <PoodlePill tone={getProjectStatusTone(project.status)} appearance="badge" size="lg">
          {statusLabel}
        </PoodlePill>
      </DetailMetaItem>
    </DetailMeta>

    {#snippet actions()}
      <PoodleButton type="button" variant="secondary" on:click={handleEdit}>
        <Pencil size={16} />
        Edit
      </PoodleButton>
      <ConfirmAction
        title="Delete Project"
        description={`Are you sure you want to delete "${project.name}"? All tasks within this project will also be deleted.`}
        confirmLabel="Delete"
        triggerLabel="Delete"
        triggerVariant="danger"
        onConfirm={handleDelete}
      />
    {/snippet}
  </PageHeader>

  <DetailsCard>
    <DetailsSection legend="Details">
      <DetailsItem label="Progress">
        <div class="progress-cell">
          <span>{completedTasks}/{tasks.length} tasks</span>
          {#if tasks.length > 0}
            <ProgressBar value={progress} max={100} />
          {/if}
        </div>
      </DetailsItem>
      {#if project.description}
        <DetailsItem label="Description" value={project.description} span="full" />
      {/if}
    </DetailsSection>

    <DetailsSection legend="Metadata">
      <DetailsItem label="Category" value={project.categoryId ? project.categoryId : "None"} />
      <DetailsItem label="Created">
        <TimeAgo date={project.createdAt} tooltipFormat="datetime" />
      </DetailsItem>
      <DetailsItem label="Updated">
        <TimeAgo date={project.updatedAt} tooltipFormat="datetime" />
      </DetailsItem>
    </DetailsSection>
  </DetailsCard>

  <section class="tasks-section">
    <div class="tasks-header">
      <h2>Tasks</h2>
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
      <FilterBar title="Filter tasks" onRefresh={() => pageData.refetch()}>
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
          <OrderBy fields={sortFields} value={orderBy} onChange={handleSortChange} />
        </PoodleField>
      </FilterBar>
    {/if}

    {#if tasks.length === 0}
      <p class="empty-state">No tasks yet. Add your first task to get started.</p>
    {:else if isTaskReorderMode}
      <ReorderableList
        controller={reorderController}
        oncancel={exitTaskReorderMode}
        onsuccess={handleTaskReorderSuccess}
        onsubmiterror={handleTaskReorderError}
      >
        {#snippet item(task)}
          <ListCard
            title={task.title}
            variant="compact"
            showDragHandle
          >
            {#snippet media()}
              <CheckSquare size={16} />
            {/snippet}
            {#snippet titleSuffix()}
              <PoodlePill
                tone={task.status === "completed" ? "success" : "neutral"}
                appearance="badge"
                size="sm"
              >
                {task.status === "completed" ? "Done" : task.status === "in_progress" ? "In Progress" : "Pending"}
              </PoodlePill>
            {/snippet}
          </ListCard>
        {/snippet}
      </ReorderableList>
    {:else}
      <ListGrid minItemWidth={24}>
        {#each tasks as task}
          <ListCard
            title={task.title}
            href={isTaskSelectionMode ? undefined : `/projects/${project.id}/tasks/${task.id}`}
            variant="compact"
          >
            {#snippet media()}
              {#if isTaskSelectionMode}
                <label class="task-checkbox">
                  <input
                    type="checkbox"
                    checked={selectedTaskIds.has(task.id)}
                    onchange={(e) => handleTaskSelectionChange(task.id, (e.target as HTMLInputElement).checked)}
                  />
                </label>
              {:else}
                <CheckSquare size={16} />
              {/if}
            {/snippet}
            {#snippet titleSuffix()}
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
            {/snippet}
          </ListCard>
        {/each}
      </ListGrid>
    {/if}

    <BatchActionBar
      selectedCount={selectedTaskIds.size}
      totalCount={tasks.length}
      loading={batchLoading}
      showStatusUpdate={true}
      statusOptions={taskStatusOptions}
      onClearSelection={clearTaskSelection}
      onSelectAll={selectAllTasks}
      onBatchDelete={handleBatchDeleteTasks}
      onBatchStatusUpdate={handleBatchUpdateTaskStatus}
    />
  </section>
{:else}
  <PoodleCallout tone="danger" message="Project not found" announceMode="polite" />
{/if}

<style>
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

  .task-checkbox {
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }

  .task-checkbox input[type="checkbox"] {
    width: 1.125rem;
    height: 1.125rem;
    accent-color: var(--accent-color, #6366f1);
    cursor: pointer;
  }
</style>
