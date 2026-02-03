<script lang="ts">
  import type { PageData } from "./$types";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { adminCommands, type Project, type TaskWithLabels, TaskStatus, TaskPriority } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import { useAuthenticatedData, PageHeader, useToasts, Banner, ReorderableList, createReorderController, FilterBar } from "@decodelabs/underlay/patterns";
  import { Button, PageLoading, FormError, ConfirmAction, Badge, ListGrid, ListCard, ProgressBar, Field, Select, OrderBy, type OrderByValue } from "@decodelabs/underlay/components";
  import { gotoWithContext, parseQueryParams } from "@decodelabs/underlay/client";
  import { BatchActionBar } from "$lib/components";
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
      getToken: () => auth.getToken(),
      defaultValue: { project: null as Project | null, tasks: [] as TaskWithLabels[] },
      onSuccess: () => {
        previousUrl = $page.url.search;
      }
    }
  );

  // Trigger fetch when auth is ready
  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

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

  type BadgeVariant = "default" | "success" | "warning" | "danger" | "info" | "muted";
  const statusVariant = $derived<BadgeVariant>(project ? ({
    active: "success",
    archived: "muted",
    on_hold: "warning"
  } as Record<string, BadgeVariant>)[project.status] ?? "default" : "default");

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
  <FormError message={pageData.error} />
{:else if project}
  <PageHeader
    title={project.name}
    backHref="/projects"
    backLabel="Back to projects"
  >
    {#snippet actions()}
      <Button type="button" variant="secondary" onclick={handleEdit}>
        <Pencil size={16} />
        Edit
      </Button>
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

  {#if project.status === "archived"}
    <Banner variant="warning" message="This project is archived and tasks cannot be modified." />
  {:else if project.status === "on_hold"}
    <Banner variant="info" message="This project is on hold." />
  {/if}

  <div class="detail-grid">
    <section class="detail-section">
      <h2>Details</h2>
      <dl class="detail-list">
        <div class="detail-item">
          <dt>Status</dt>
          <dd>
            <Badge variant={statusVariant}>{statusLabel}</Badge>
          </dd>
        </div>
        <div class="detail-item">
          <dt>Progress</dt>
          <dd class="progress-cell">
            <span>{completedTasks}/{tasks.length} tasks</span>
            {#if tasks.length > 0}
              <ProgressBar value={progress} max={100} />
            {/if}
          </dd>
        </div>
        {#if project.description}
          <div class="detail-item full">
            <dt>Description</dt>
            <dd>{project.description}</dd>
          </div>
        {/if}
      </dl>
    </section>

    <section class="detail-section">
      <h2>Metadata</h2>
      <dl class="detail-list">
        <div class="detail-item">
          <dt>ID</dt>
          <dd><code>{project.id}</code></dd>
        </div>
        <div class="detail-item">
          <dt>Category</dt>
          <dd>{project.categoryId ? project.categoryId : "None"}</dd>
        </div>
        <div class="detail-item">
          <dt>Created</dt>
          <dd>{new Date(project.createdAt).toLocaleString()}</dd>
        </div>
        <div class="detail-item">
          <dt>Updated</dt>
          <dd>{new Date(project.updatedAt).toLocaleString()}</dd>
        </div>
      </dl>
    </section>
  </div>

  <section class="tasks-section">
    <div class="tasks-header">
      <h2>Tasks</h2>
      <div class="tasks-header-actions">
        {#if tasks.length > 1 && !isTaskSelectionMode}
          <Button
            type="button"
            variant={isTaskReorderMode ? "danger" : "subtle"}
            size="sm"
            onclick={() => isTaskReorderMode ? exitTaskReorderMode() : enterTaskReorderMode()}
          >
            <ArrowUpDown size={14} />
            Reorder
          </Button>
        {/if}
        {#if tasks.length > 0 && !isTaskReorderMode}
          <Button
            type="button"
            variant={isTaskSelectionMode ? "danger" : "subtle"}
            size="sm"
            onclick={toggleTaskSelectionMode}
          >
            <CheckSquare2 size={14} />
            {isTaskSelectionMode ? "Cancel" : "Select"}
          </Button>
        {/if}
        {#if !isTaskSelectionMode && !isTaskReorderMode}
          <Button type="button" variant="primary" size="sm" onclick={handleAddTask}>
            <Plus size={14} />
            Add Task
          </Button>
        {/if}
      </div>
    </div>

    {#if !isTaskReorderMode}
      <FilterBar title="Filter tasks" onRefresh={() => pageData.refetch()}>
        <Field label="Status" forId="taskStatus">
          <Select
            id="taskStatus"
            value={selectedStatus}
            onchange={handleStatusChange}
            items={statusItems}
            placeholder="All statuses"
            clearable
            defaultValue="All"
          />
        </Field>
        <Field label="Priority" forId="taskPriority">
          <Select
            id="taskPriority"
            value={selectedPriority}
            onchange={handlePriorityChange}
            items={priorityItems}
            placeholder="All priorities"
            clearable
            defaultValue="All"
          />
        </Field>
        <Field label="Sort" forId="taskSort">
          <OrderBy fields={sortFields} value={orderBy} onChange={handleSortChange} />
        </Field>
      </FilterBar>
    {/if}

    {#if tasks.length === 0}
      <p class="empty-state">No tasks yet. Add your first task to get started.</p>
    {:else if isTaskReorderMode}
      <ReorderableList controller={reorderController} oncancel={exitTaskReorderMode} onsuccess={handleTaskReorderSuccess}>
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
              <Badge variant={task.status === "completed" ? "success" : task.status === "in_progress" ? "info" : "muted"} size="sm">
                {task.status === "completed" ? "Done" : task.status === "in_progress" ? "In Progress" : "Pending"}
              </Badge>
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
              <Badge variant={task.status === "completed" ? "success" : task.status === "in_progress" ? "info" : "muted"} size="sm">
                {task.status === "completed" ? "Done" : task.status === "in_progress" ? "In Progress" : "Pending"}
              </Badge>
              <Badge variant={task.priority === "urgent" ? "danger" : task.priority === "high" ? "warning" : "muted"} size="sm">
                {task.priority}
              </Badge>
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
  <FormError message="Project not found" />
{/if}

<style>
  .detail-grid {
    display: grid;
    gap: 2rem;
    margin-top: 1.5rem;
  }

  .detail-section {
    background: var(--bg-surface, #fff);
    border: 1px solid var(--border-color, #e5e7eb);
    border-radius: 0.5rem;
    padding: 1.5rem;
  }

  .detail-section h2 {
    margin: 0 0 1rem;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary, #111827);
  }

  .detail-list {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
    margin: 0;
  }

  .detail-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .detail-item.full {
    grid-column: span 2;
  }

  .detail-item dt {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-secondary, #6b7280);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .detail-item dd {
    margin: 0;
    font-size: 0.875rem;
    color: var(--text-primary, #111827);
  }

  .progress-cell {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  code {
    font-family: monospace;
    font-size: 0.8em;
    background: var(--bg-muted, #f3f4f6);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }

  .tasks-section {
    margin-top: 2rem;
    background: var(--bg-surface, #fff);
    border: 1px solid var(--border-color, #e5e7eb);
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
