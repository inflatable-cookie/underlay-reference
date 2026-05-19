<script lang="ts">
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import type { QueryParams } from "@decodelabs/underlay/client/query";
  import { EntityListPage, toPagedListResult } from "@decodelabs/underlay/templates";
  import { useToasts } from "@decodelabs/underlay/runtime/feedback";
  import {
    Pill as PoodlePill,
    TimeAgo,
    type TableColumn,
    type TableRow,
    type TableRowAction
  } from "@poodle/svelte";
  import {
    adminCommands,
    type User,
    UserRole as UserRoleConst,
    UserStatus as UserStatusConst
  } from "@api-client";
  import { getUserRoleTone, getUserStatusTone } from "$lib/utils/accents";

  interface Props {
    title?: string;
    hideTitle?: boolean;
    subtitle?: string;
    eyebrow?: string;
    headerLevel?: 1 | 2 | 3 | 4 | 5 | 6;
    backHref?: string;
    backLabel?: string;
    query: QueryParams;
    onQueryChange: (query: QueryParams) => void;
  }

  let {
    title = "Users",
    hideTitle = false,
    subtitle,
    eyebrow,
    headerLevel = 2,
    backHref = "/",
    backLabel = "Back to dashboard",
    query,
    onQueryChange
  }: Props = $props();

  const toastStore = useToasts();

  const columns: TableColumn[] = [
    { id: "email", label: "Email", width: "2fr" },
    { id: "displayName", label: "Display Name", width: "1.5fr" },
    { id: "role", label: "Role", width: "120px" },
    { id: "status", label: "Status", width: "100px" },
    { id: "createdAt", label: "Created", width: "100px", hideOnMobile: true }
  ];

  const filters = [
    {
      id: "query",
      type: "search" as const,
      label: "Search",
      placeholder: "Search by email or display name..."
    },
    {
      id: "role",
      type: "select" as const,
      label: "Role",
      options: [
        { value: "All", label: "All roles" },
        { value: UserRoleConst.User, label: "User" },
        { value: UserRoleConst.Tester, label: "Tester" },
        { value: UserRoleConst.Editor, label: "Editor" },
        { value: UserRoleConst.Admin, label: "Admin" },
        { value: UserRoleConst.Support, label: "Support" },
        { value: UserRoleConst.Superadmin, label: "Superadmin" }
      ]
    },
    {
      id: "status",
      type: "select" as const,
      label: "Status",
      options: [
        { value: "All", label: "All statuses" },
        { value: UserStatusConst.Active, label: "Active" },
        { value: UserStatusConst.Suspended, label: "Suspended" },
        { value: UserStatusConst.Deleted, label: "Deleted" }
      ]
    },
    {
      id: "sort",
      type: "sort" as const,
      label: "Sort",
      sortFields: [
        { key: "createdAt", label: "Created", defaultDirection: "desc" as const },
        { key: "email", label: "Email" },
        { key: "displayName", label: "Display name" },
        { key: "role", label: "Role" },
        { key: "status", label: "Status" }
      ]
    }
  ];

  async function dataLoader(fetch: typeof window.fetch, token: string | null, nextQuery: QueryParams) {
    if (!token) throw new Error("Not authenticated");
    const response = await adminCommands.listUsers(fetch, token, nextQuery);
    return toPagedListResult(response);
  }

  function getRowActions(_row: TableRow<User>): TableRowAction[] {
    return [
      { value: "edit", label: "Edit" },
      { value: "copy-id", label: "Copy ID" },
      { value: "copy-email", label: "Copy Email" }
    ];
  }

  async function copyText(text: string): Promise<void> {
    try {
      await globalThis.navigator?.clipboard?.writeText(text);
      toastStore.push({ variant: "success", message: "Copied to clipboard" });
      return;
    } catch {
      // Fall through to legacy copy path.
    }

    try {
      const doc = globalThis.document;
      if (!doc) throw new Error("No document");
      const textarea = doc.createElement("textarea");
      textarea.value = text;
      textarea.style.position = "fixed";
      textarea.style.opacity = "0";
      doc.body.appendChild(textarea);
      textarea.select();
      doc.execCommand("copy");
      textarea.remove();
      toastStore.push({ variant: "success", message: "Copied to clipboard" });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to copy";
      toastStore.push({ variant: "error", message });
    }
  }

  function handleRowActionSelect(row: TableRow<User>, action: TableRowAction) {
    const user = row.data;
    if (!user) return;

    switch (action.value) {
      case "edit":
        void gotoWithContext(`/users/${user.id}/edit`, {
          label: title,
          href: backHref,
          type: "list"
        });
        break;
      case "copy-id":
        void copyText(user.id);
        break;
      case "copy-email":
        void copyText(user.email);
        break;
    }
  }

  function handleAddUser() {
    void gotoWithContext("/users/new", {
      label: title,
      href: backHref,
      type: "list"
    });
  }
</script>

{#snippet renderCell(column: TableColumn, row: TableRow<User>, value: string)}
  {@const user = row.data}
  {#if column.id === "email" && user}
    <a href={`/users/${user.id}`} class="email-link">{value}</a>
  {:else if column.id === "role" && user}
    <PoodlePill tone={getUserRoleTone(user.role)} appearance="badge" size="sm">{user.role}</PoodlePill>
  {:else if column.id === "status" && user}
    <PoodlePill tone={getUserStatusTone(user.status)} appearance="badge" size="sm">{user.status}</PoodlePill>
  {:else if column.id === "createdAt" && user}
    <TimeAgo datetime={user.createdAt} tooltipFormat="datetime" short />
  {:else}
    {value || "—"}
  {/if}
{/snippet}

<EntityListPage
  {title}
  {hideTitle}
  {subtitle}
  {eyebrow}
  {headerLevel}
  {backHref}
  {backLabel}
  {dataLoader}
  presentation="table"
  {columns}
  {filters}
  rowActions={getRowActions}
  renderCell={renderCell as never}
  onRowActionSelect={handleRowActionSelect}
  {query}
  {onQueryChange}
  onAdd={handleAddUser}
  addLabel="Add user"
/>

<style>
  .email-link {
    color: inherit;
    text-decoration: none;
    font-weight: 500;
  }

  .email-link:hover {
    text-decoration: underline;
  }
</style>
