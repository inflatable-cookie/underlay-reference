<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { PageHeader, useToasts, useAuthenticatedData, FilterBar } from "@decodelabs/underlay/patterns";
  import {
    DropdownMenu,
    Field,
    FormError,
    ListCard,
    ListGrid,
    PageLoading,
    Select,
    TimeAgo
  } from "@decodelabs/underlay/components";
  import Calendar from "lucide-svelte/icons/calendar";
  import { adminCommands } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
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
      getToken: () => auth.getToken(),
      defaultValue: { tasks: [] as ScheduledTaskSummary[] },
      onSuccess: () => {
        previousUrl = $page.url.search;
      }
    }
  );

  // Trigger fetch when auth is ready
  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  // Refetch when URL changes (for filtering)
  $effect(() => {
    const currentUrl = $page.url.search;
    if (previousUrl !== null && previousUrl !== currentUrl) {
      previousUrl = currentUrl;
      pageData.refetch();
    }
  });

  const tasks = $derived(pageData.data?.tasks ?? []);

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
</script>

<section class="scheduled-tasks-page">
  <PageHeader section="Scheduled Tasks" backHref="/system" backLabel="Back to system">
    Manage cron-scheduled maintenance tasks. Tasks run automatically based on their schedule.
  </PageHeader>

  <FilterBar title="Filters" startCollapsed={true} onRefresh={() => pageData.refetch()}>
    <Field label="Status">
      <Select
        name="enabled"
        value={filters.enabled}
        items={enabledOptions}
        placeholder="All tasks"
        clearable
        defaultValue=""
        onchange={(value) => {
          const url = new URL($page.url);
          if (value) {
            url.searchParams.set("enabled", value);
          } else {
            url.searchParams.delete("enabled");
          }
          void goto(url.toString());
        }}
      />
    </Field>
  </FilterBar>

  {#if pageData.loading}
    <PageLoading message="Loading scheduled tasks..." />
  {:else if pageData.error}
    <FormError message={pageData.error} />
  {:else if tasks.length === 0}
    <p class="scheduled-tasks-page__empty">
      No scheduled tasks found for the current filters.
    </p>
  {:else}
    <ListGrid minItemWidth={26}>
      {#each tasks as task}
        {@const href = `/system/scheduled-tasks/${encodeURIComponent(task.id)}`}
        {@const menuItems = [
          {
            label: "Trigger now",
            onSelect: () => handleTrigger(task)
          },
          {
            label: task.enabled ? "Disable task" : "Enable task",
            onSelect: () => handleToggle(task)
          }
        ]}
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
            <DropdownMenu items={menuItems} {align}>
              {#snippet trigger()}
                {@render mediaContent()}
              {/snippet}
            </DropdownMenu>
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
  {/if}
</section>

<style>
  .scheduled-tasks-page {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .scheduled-tasks-page__empty {
    margin: 0;
    font-size: 0.9rem;
    color: var(--admin-color-text-muted);
  }
</style>
