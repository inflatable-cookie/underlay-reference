<script lang="ts">
  import type { QueryParams } from "@decodelabs/underlay/client/query";
  import { EntityListPage, toPagedListResult } from "@decodelabs/underlay/templates";
  import type { AuditLogEntry, LogActor } from "@poodle/svelte";
  import { adminCommands, type ActivityEntry } from "@api-client";

  interface Props {
    title?: string;
    backHref?: string;
    backLabel?: string;
    query: QueryParams;
    onQueryChange: (query: QueryParams) => void;
  }

  let {
    title = "Audit Log",
    backHref = "/system",
    backLabel = "Back to system",
    query,
    onQueryChange
  }: Props = $props();

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

  function getFilterValue(nextQuery: QueryParams, field: string): string | undefined {
    const filter = nextQuery.filters?.find((entry) => entry.field === field);
    if (!filter || filter.value === "" || filter.value === "All") {
      return undefined;
    }
    return filter.value;
  }

  async function dataLoader(fetch: typeof window.fetch, token: string | null, nextQuery: QueryParams) {
    if (!token) throw new Error("Not authenticated");

    const response = await adminCommands.listActivity(fetch, token, {
      action: getFilterValue(nextQuery, "action"),
      resourceType: getFilterValue(nextQuery, "resourceType"),
      page: nextQuery.page ?? 1,
      limit: nextQuery.limit ?? 30
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
  {title}
  {backHref}
  {backLabel}
  {dataLoader}
  presentation="log"
  {filters}
  {query}
  {onQueryChange}
  {toLogEntries}
  {getActorHref}
  {getResourceHref}
  {formatAction}
  {formatResourceType}
/>
