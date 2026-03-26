<script lang="ts">
  import { Callout as PoodleCallout } from "@poodle/svelte-primitives";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { useToasts, useAuthenticatedData } from "@decodelabs/underlay/patterns";
  import {
        ListCard,
    ListGrid,
    TimeAgo
  } from "@decodelabs/underlay/components";
  import {
    FilterToolbar,
    ListContainer
  } from "@poodle/svelte-composites";
  import {
    Field as PoodleField,
    IconButton as PoodleIconButton,
    Menu as PoodleMenu,
    Select as PoodleSelect,
    type MenuItem
  } from "@poodle/svelte-primitives";
  import Calendar from "lucide-svelte/icons/calendar";
  import { adminCommands } from "acme-client";
  import { auth } from "$lib/stores/auth";
  import type { ScheduledTaskSummary } from "acme-client";

  const toastStore = useToasts();

  // Track URL for refetching when filters change
  let previousUrl = $state<string | null>(null);

  // Derive filters from URL
  const filters = $derived({
    enabled: $page.url.searchParams.get("enabled") ?? ""
  });

  // Fetch scheduled tasks using authenticated data pattern
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const enabledParam = $page.url.searchParams.get("enabled");
      const enabled = enabledParam === "true" ? true : enabledParam === "false" ? false : undefined;

      const tasks = await adminCommands.listScheduledTasks(fetch, token, {
        enabled,
        limit: 100
      });
      return { tasks };
    },
    {
      defaultValue: { tasks: [] as ScheduledTaskSummary[] },
      onSuccess: () => {
        previousUrl = $page.url.search;
      }
    }
  );

  // Refetch when URL changes (for filtering)
  $effect(() => {
    const currentUrl = $page.url.search;
    if (previousUrl !== null && previousUrl !== currentUrl) {
      previousUrl = currentUrl;
      pageData.refetch();
    }
  });

  const tasks = $derived(pageData.data?.tasks ?? []);
  let filtersCollapsed = $state(true);
  const listState = $derived(
    pageData.loading ? "loading" : pageData.error ? "error" : tasks.length === 0 ? "empty" : "ready"
  );
  const filterSummaryText = $derived(
    tasks.length > 0 ? `Showing ${tasks.length} scheduled task${tasks.length === 1 ? "" : "s"}` : "No scheduled tasks"
  );

  /** Convert snake_case task name to human-readable title */
  function formatTaskName(name: string): string {
    return name
      .split("_")
      .map((word, i) => i === 0 ? word.charAt(0).toUpperCase() + word.slice(1) : word)
      .join(" ");
  }

  async function handleToggle(task: ScheduledTaskSummary) {
    const token = auth.getToken();
    if (!token) return;

    try {
      await adminCommands.toggleScheduledTask(task.id, !task.enabled, fetch, token);
      toastStore.push({ variant: "success", message: task.enabled ? "Task disabled" : "Task enabled" });
      pageData.refetch();
    } catch (err) {
      toastStore.push({ variant: "error", message: "Failed to toggle task" });
    }
  }

  async function handleTrigger(task: ScheduledTaskSummary) {
    const token = auth.getToken();
    if (!token) return;

    try {
      const result = await adminCommands.triggerScheduledTask(task.id, fetch, token);
      toastStore.push({ variant: "success", message: "Job created" });
      // Navigate to the new job
      goto(`/system/jobs/${result.jobId}`);
    } catch (err) {
      toastStore.push({ variant: "error", message: "Failed to trigger task" });
    }
  }

  const enabledOptions = [
    { value: "", label: "All tasks" },
    { value: "true", label: "Enabled only" },
    { value: "false", label: "Disabled only" }
  ];

  function getMenuItems(task: ScheduledTaskSummary): MenuItem[] {
    return [
      { value: "trigger", label: "Trigger now" },
      { value: "toggle", label: task.enabled ? "Disable task" : "Enable task" }
    ];
  }

  function handleMenuAction(task: ScheduledTaskSummary, value: string) {
    if (value === "trigger") {
      void handleTrigger(task);
      return;
    }

    if (value === "toggle") {
      void handleToggle(task);
    }
  }
</script>

<section class="scheduled-tasks-page">
  <ListContainer
    title="Scheduled Tasks"
    subtitle="Manage cron-scheduled maintenance tasks. Tasks run automatically based on their schedule."
    eyebrow="System"
    state={listState}
    loadingMessage="Loading scheduled tasks..."
    emptyTitle="No scheduled tasks"
    emptyMessage="No scheduled tasks found for the current filters."
    showPagination={false}
  >
    <svelte:fragment slot="filters">
      <FilterToolbar
        ariaLabel="Scheduled task filters"
        columns={1}
        collapsible
        bind:collapsed={filtersCollapsed}
        summaryText={filterSummaryText}
      >
        <svelte:fragment slot="actions">
          <PoodleIconButton
            icon="refresh-cw"
            variant="secondary"
            size="sm"
            ariaLabel="Refresh tasks"
            tooltip="Refresh tasks"
            on:click={() => pageData.refetch()}
          />
        </svelte:fragment>

        <PoodleField id="scheduled-task-status-filter" label="Status" let:describedBy>
          <PoodleSelect
            id="scheduled-task-status-filter"
            name="enabled"
            value={filters.enabled}
            describedBy={describedBy}
            options={enabledOptions}
            on:valueChange={(event) => {
              const value = event.detail.value;
              const url = new URL($page.url);
              if (value) {
                url.searchParams.set("enabled", value);
              } else {
                url.searchParams.delete("enabled");
              }
              void goto(url.toString());
            }}
          />
        </PoodleField>
      </FilterToolbar>
    </svelte:fragment>

    <svelte:fragment slot="error">
      <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
    </svelte:fragment>

    <ListGrid minItemWidth={26}>
      {#each tasks as task}
        {@const href = `/system/scheduled-tasks/${encodeURIComponent(task.id)}`}
        <ListCard
          href={href}
          title={formatTaskName(task.name)}
          subtitle={task.schedule}
          isLive={task.enabled}
        >
          {#snippet media()}
            <Calendar size={30} />
          {/snippet}

          {#snippet actions({ trigger: mediaContent, align })}
            <PoodleMenu items={getMenuItems(task)} placement={align === "end" ? "bottom-end" : "bottom-start"} on:action={(event) => handleMenuAction(task, event.detail.value)}>
              <div slot="trigger">
                {@render mediaContent()}
              </div>
            </PoodleMenu>
          {/snippet}

          <span>
            {#if task.lastCompletedAt}
              <TimeAgo date={task.lastCompletedAt} />
            {:else}
              Never run
            {/if}
          </span>
        </ListCard>
      {/each}
    </ListGrid>
  </ListContainer>
</section>

<style>
  .scheduled-tasks-page {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

</style>
