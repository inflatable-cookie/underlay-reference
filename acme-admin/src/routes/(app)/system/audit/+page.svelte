<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { PageHeader, useAuthenticatedData } from "@decodelabs/underlay/patterns";
  import { LogList, type LogEntry, type LogFilter, type LogActor } from "@decodelabs/underlay/components";
  import { adminCommands } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";

  // Track URL for refetching when filters change
  let previousUrl = $state<string | null>(null);

  // Derive filters from URL
  const filterValues = $derived({
    action: $page.url.searchParams.get("action") ?? "",
    resourceType: $page.url.searchParams.get("resourceType") ?? ""
  });

  // Filter configuration
  const filters: LogFilter[] = [
    {
      field: "action",
      label: "Action",
      type: "select",
      placeholder: "All actions",
      options: [
        { value: "create", label: "Create" },
        { value: "update", label: "Update" },
        { value: "delete", label: "Delete" },
        { value: "restore", label: "Restore" }
      ]
    },
    {
      field: "resourceType",
      label: "Resource",
      type: "select",
      placeholder: "All resources",
      options: [
        { value: "category", label: "Category" },
        { value: "project", label: "Project" },
        { value: "task", label: "Task" },
        { value: "label", label: "Label" },
        { value: "user", label: "User" }
      ]
    }
  ];

  // Fetch audit log using authenticated data pattern
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const response = await adminCommands.listActivity(fetch, token, {
        limit: 100
      });
      return { entries: response.data };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { entries: [] },
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

  // Transform activity entries to LogEntry format
  const logEntries = $derived<LogEntry[]>(
    (pageData.data?.entries ?? [])
      .filter((entry) => {
        // Client-side filtering since API doesn't support action/resourceType filters
        const actionFilter = filterValues.action;
        const resourceTypeFilter = filterValues.resourceType;

        if (actionFilter && entry.action !== actionFilter) return false;
        if (resourceTypeFilter && entry.resourceType !== resourceTypeFilter) return false;
        return true;
      })
      .map((entry) => ({
        id: entry.id,
        occurredAt: entry.occurredAt,
        actor: entry.actor
          ? {
              id: entry.actor.id,
              name: entry.actor.displayName ?? undefined,
              email: entry.actor.email
            }
          : null,
        action: entry.action,
        resourceType: entry.resourceType,
        resourceId: entry.resourceId
      }))
  );

  // Handle filter changes - update URL
  function handleFilterChange(field: string, value: string) {
    const url = new URL($page.url);

    if (value && value.trim()) {
      url.searchParams.set(field, value);
    } else {
      url.searchParams.delete(field);
    }

    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  // Clear all filters
  function handleClearFilters() {
    goto("/system/audit", { replaceState: true, keepFocus: true });
  }

  // Get link to user detail page
  function getActorHref(actor: LogActor): string {
    return `/users/${actor.id}`;
  }

  // Get link to resource detail page (don't link if deleted)
  function getResourceHref(resourceType: string, resourceId: string, action: string): string | null {
    // Don't link to deleted resources
    if (action === "delete" || action === "soft_delete") {
      return null;
    }

    switch (resourceType) {
      case "category":
        return `/categories/${resourceId}`;
      case "project":
        return `/projects/${resourceId}`;
      case "task":
        // Tasks are nested under projects, so we can't easily link to them
        return null;
      case "label":
        // Labels are nested under projects
        return null;
      case "user":
        return `/users/${resourceId}`;
      default:
        return null;
    }
  }
</script>

<section class="audit-page">
  <PageHeader section="Audit Log" backHref="/system" backLabel="Back to system">
    Track changes made to content and configuration across the platform.
  </PageHeader>

  <LogList
    entries={logEntries}
    loading={pageData.loading}
    error={pageData.error}
    emptyMessage="No audit log entries found for the current filters."
    {filters}
    {filterValues}
    onFilterChange={handleFilterChange}
    onClearFilters={handleClearFilters}
    onRefresh={() => pageData.refetch()}
    {getActorHref}
    {getResourceHref}
  />
</section>

<style>
  .audit-page {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
</style>
