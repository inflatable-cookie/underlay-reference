<script lang="ts">
  import type { QueryParams } from "@decodelabs/underlay/client/query";
  import { SystemAuditLogListPage, toPagedListResult } from "@decodelabs/underlay/templates";
  import type { SystemAuditLogEntry } from "@decodelabs/underlay/templates";
  import type { LogActor } from "@poodle/svelte";
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

  const resourceOptions = [
    { value: "All", label: "All resources" },
    { value: "category", label: "Category" },
    { value: "project", label: "Project" },
    { value: "task", label: "Task" },
    { value: "label", label: "Label" },
    { value: "user", label: "User" }
  ];

  function toAuditEntry(entry: ActivityEntry): SystemAuditLogEntry {
    return {
      id: entry.id,
      occurredAt: entry.occurredAt,
      actor: entry.actor
        ? { id: entry.actor.id, email: entry.actor.email, name: entry.actor.displayName ?? undefined }
        : null,
      action: entry.action,
      resourceType: entry.resourceType,
      resourceId: entry.resourceId,
      resourceLabel: (entry.details?.resourceLabel as string | undefined) ?? undefined,
      details: entry.details
    };
  }

  function getActorHref(actor: LogActor): string {
    return `/users/${actor.id}`;
  }

  function getResourceHref(resourceType: string, resourceId: string, action: string): string | null {
    if (action === "delete" || action === "soft_delete") return null;
    if (resourceType === "category") return `/categories/${resourceId}`;
    if (resourceType === "project") return `/projects/${resourceId}`;
    if (resourceType === "user") return `/users/${resourceId}`;
    return null;
  }
</script>

<SystemAuditLogListPage
  {title}
  {backHref}
  {backLabel}
  {query}
  {onQueryChange}
  {resourceOptions}
  dataLoader={async (fetch, token, request) => {
    const response = await adminCommands.listActivity(fetch, token, {
      action: request.action,
      resourceType: request.resourceType,
      page: request.page,
      limit: request.limit
    });
    const result = toPagedListResult<ActivityEntry>(response);
    return { ...result, data: result.data.map(toAuditEntry) };
  }}
  {getActorHref}
  {getResourceHref}
/>
