<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { EntityListPage, toPagedListResult } from "@decodelabs/underlay/templates";
  import {
    buildQueryString,
    parseQueryParams,
    type QueryParams
  } from "@decodelabs/underlay/client/query";
  import type { AuditLogEntry, LogActor } from "@poodle/svelte";
  import { adminCommands, type ActivityEntry } from "@api-client";

  const currentQuery = $derived(parseQueryParams($page.url.searchParams));

  const filters = [
    {
      id: "action",
      type: "select" as const,
      label: "Action",
      options: [
        { value: "All", label: "All actions" },
        { value: "create", label: "Create" },
        { value: "update", label: "Update" },
        { value: "delete", label: "Delete" },
        { value: "restore", label: "Restore" }
      ]
    },
    {
      id: "resourceType",
      type: "select" as const,
      label: "Resource",
      options: [
        { value: "All", label: "All resources" },
        { value: "category", label: "Category" },
        { value: "project", label: "Project" },
        { value: "task", label: "Task" },
        { value: "label", label: "Label" },
        { value: "user", label: "User" }
      ]
    }
  ];

  function updateUrl(nextQuery: QueryParams) {
    const url = new URL($page.url);
    url.search = buildQueryString(nextQuery);
    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  function getFilterValue(query: QueryParams, field: string): string | undefined {
    const filter = query.filters?.find((entry) => entry.field === field);
    if (!filter || filter.value === "" || filter.value === "All") {
      return undefined;
    }
    return filter.value;
  }

  async function dataLoader(fetch: typeof window.fetch, token: string | null, query: QueryParams) {
    if (!token) throw new Error("Not authenticated");

    const response = await adminCommands.listActivity(fetch, token, {
      action: getFilterValue(query, "action"),
      resourceType: getFilterValue(query, "resourceType"),
      page: query.page ?? 1,
      limit: query.limit ?? 30
    });
    return toPagedListResult(response);
  }

  function toLogEntries(entries: ActivityEntry[]): AuditLogEntry[] {
    return entries.map((entry) => ({
      id: entry.id,
      occurredAt: entry.occurredAt,
      actor: entry.actor
        ? {
            id: entry.actor.id,
            email: entry.actor.email,
            name: entry.actor.displayName ?? undefined
          }
        : null,
      action: entry.action,
      resourceType: entry.resourceType,
      resourceId: entry.resourceId,
      resourceLabel: (entry.details?.resourceLabel as string | undefined) ?? undefined,
      details: entry.details
    }));
  }

  function getActorHref(actor: LogActor): string {
    return `/users/${actor.id}`;
  }

  function getResourceHref(resourceType: string, resourceId: string, action: string): string | null {
    if (action === "delete" || action === "soft_delete") {
      return null;
    }

    switch (resourceType) {
      case "category":
        return `/categories/${resourceId}`;
      case "project":
        return `/projects/${resourceId}`;
      case "user":
        return `/users/${resourceId}`;
      default:
        return null;
    }
  }

  function formatAction(action: string): string {
    return action.charAt(0).toUpperCase() + action.slice(1);
  }

  function formatResourceType(resourceType: string): string {
    return resourceType.charAt(0).toUpperCase() + resourceType.slice(1);
  }
</script>

<EntityListPage
  title="Audit Log"
  backHref="/system"
  backLabel="Back to system"
  {dataLoader}
  presentation="log"
  {filters}
  query={currentQuery}
  onQueryChange={updateUrl}
  {toLogEntries}
  {getActorHref}
  {getResourceHref}
  {formatAction}
  {formatResourceType}
/>
