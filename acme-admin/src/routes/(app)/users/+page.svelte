<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { EntityListPage, toPagedListResult } from "@decodelabs/underlay/templates";
  import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import { buildQueryString, parseQueryParams } from "@decodelabs/underlay/client/query";
  import type { QueryParams } from "@decodelabs/underlay/client/query";
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
    type UserRole,
    type UserStatus,
    UserRole as UserRoleConst,
    UserStatus as UserStatusConst
  } from "@api-client";
  import { getUserRoleTone, getUserStatusTone } from "$lib/utils/accents";

  const toastStore = useToasts();
  const currentQuery = $derived(parseQueryParams($page.url.searchParams));

  const columns: TableColumn[] = [
    {
      id: "email",
      label: "Email",
      width: "2fr"
    },
    {
      id: "displayName",
      label: "Display Name",
      width: "1.5fr"
    },
    {
      id: "role",
      label: "Role",
      width: "120px"
    },
    {
      id: "status",
      label: "Status",
      width: "100px"
    },
    {
      id: "createdAt",
      label: "Created",
      width: "100px",
      hideOnMobile: true
    }
  ];

  const filters = [
    {
      id: "email",
      type: "search" as const,
      label: "Email",
      placeholder: "Search by email..."
    },
    {
      id: "displayName",
      type: "search" as const,
      label: "Display name",
      placeholder: "Search by display name..."
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

  function updateUrl(nextQuery: QueryParams) {
    const url = new URL($page.url);
    url.search = buildQueryString(nextQuery);
    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  async function dataLoader(fetch: typeof window.fetch, token: string | null, query: QueryParams) {
    if (!token) throw new Error("Not authenticated");
    const response = await adminCommands.listUsers(fetch, token, query);
    return toPagedListResult(response);
  }

  function getRowActions(_row: TableRow<User>): TableRowAction[] {
    return [
      { value: "edit", label: "Edit" },
      { value: "copy-id", label: "Copy ID" },
      { value: "copy-email", label: "Copy Email" }
    ];
  }

  async function copyToClipboard(text: string): Promise<void> {
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
          label: "Users",
          href: "/users",
          type: "list"
        });
        break;
      case "copy-id":
        void copyToClipboard(user.id);
        break;
      case "copy-email":
        void copyToClipboard(user.email);
        break;
    }
  }

  function handleAddUser() {
    void gotoWithContext("/users/new", {
      label: "Users",
      href: "/users",
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
  title="Users"
  backHref="/"
  backLabel="Back to dashboard"
  {dataLoader}
  presentation="table"
  {columns}
  {filters}
  rowActions={getRowActions}
  renderCell={renderCell as never}
  onRowActionSelect={handleRowActionSelect}
  query={currentQuery}
  onQueryChange={updateUrl}
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
