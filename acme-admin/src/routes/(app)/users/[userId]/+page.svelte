<script lang="ts">
  import { goto } from "$app/navigation";
  import {
    AlertDialog,
    Button,
    Card,
    Code,
    DataTable,
    DetailsCard,
    DetailsItem,
    DetailsSection,
    Dialog,
    DropdownMenu,
    Field,
    FormError,
    PageLoading,
    Pill,
    Select,
    TabsContent,
    TabsList,
    TabsRoot,
    TabsTrigger,
    type DataTableAction,
    type DataTableColumn
  } from "@decodelabs/underlay/components";
  import {
    PageHeader,
    PageHeaderMeta,
    PageHeaderMetaRow,
    PageHeaderMetaItem,
    PageHeaderMetaSeparator,
    computeBackInfo,
    consumeNavigationContext,
    useAuthenticatedData,
    useToasts
  } from "@decodelabs/underlay/patterns";
  import { gotoWithContext } from "@decodelabs/underlay/client";
  import {
    adminCommands,
    type ActivityEntry,
    type Session,
    type UserDetail,
    type UserRole,
    UserRole as UserRoleConst
  } from "@api-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import MoreVertical from "lucide-svelte/icons/more-vertical";

  interface Props {
    data: { userId: string };
  }

  let { data }: Props = $props();
  const toastStore = useToasts();

  const defaultBackHref = "/users";
  const { backInfo } = consumeNavigationContext("Back to users", defaultBackHref);

  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const [user, sessions, activity] = await Promise.all([
        adminCommands.getUser(data.userId, fetch, token),
        adminCommands.listUserSessions(data.userId, fetch, token),
        adminCommands.listActivityForUser(data.userId, fetch, token, { limit: 10 })
      ]);

      return {
        user,
        sessions,
        activity: activity.data
      };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: {
        user: null as UserDetail | null,
        sessions: [] as Session[],
        activity: [] as ActivityEntry[]
      }
    }
  );

  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  const user = $derived(pageData.data?.user ?? null);
  const sessions = $derived(pageData.data?.sessions ?? []);
  const activity = $derived(pageData.data?.activity ?? []);

  const computedBackInfo = $derived(
    computeBackInfo(
      backInfo,
      user
        ? {
            href: `/users/${user.id}`,
            label: "Back to user"
          }
        : undefined
    )
  );

  let sessionToRevoke = $state<Session | null>(null);
  let showRevokeDialog = $state(false);
  let revokingSession = $state(false);

  $effect(() => {
    showRevokeDialog = sessionToRevoke !== null;
  });

  let showRoleDialog = $state(false);
  let selectedRole = $state<UserRole>(UserRoleConst.User);
  let updatingRole = $state(false);
  let activeTab = $state("details");

  $effect(() => {
    if (user) {
      selectedRole = user.role;
    }
  });

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleString();
  }

  function getRoleAccent(role: string): string {
    switch (role) {
      case "superadmin": return "#dc2626";
      case "admin": return "#8b5cf6";
      case "support": return "#3b82f6";
      case "editor": return "#14b8a6";
      case "tutor": return "#22c55e";
      case "tester": return "#f97316";
      default: return "#64748b";
    }
  }

  function getStatusAccent(status: string): string {
    switch (status) {
      case "active": return "#22c55e";
      case "suspended": return "#f97316";
      case "deleted": return "#dc2626";
      default: return "#64748b";
    }
  }

  function getSessionStatusAccent(status: string): string {
    switch (status) {
      case "active": return "#22c55e";
      case "expired": return "#f59e0b";
      case "revoked": return "#dc2626";
      default: return "#64748b";
    }
  }

  function getActivityAccent(action: string): string {
    switch (action) {
      case "create":
      case "created":
      case "restore":
      case "restored":
      case "unsuspend":
        return "#22c55e";
      case "delete":
      case "deleted":
      case "soft_delete":
      case "suspend":
        return "#dc2626";
      case "update":
      case "updated":
      case "upload":
      case "uploaded":
      case "role_change":
        return "#3b82f6";
      case "login":
      case "logout":
        return "#64748b";
      default:
        return "#8b5cf6";
    }
  }

  function truncateUserAgent(ua: string | null | undefined, max = 70): string {
    if (!ua) return "—";
    return ua.length > max ? ua.substring(0, max) + "…" : ua;
  }

  async function copyToClipboard(text: string): Promise<void> {
    try {
      await globalThis.navigator?.clipboard?.writeText(text);
      toastStore.push({ variant: "success", message: "Copied to clipboard" });
      return;
    } catch {
      // Fall through to legacy approach
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
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to copy";
      toastStore.push({ variant: "error", message });
    }
  }

  const sessionColumns: DataTableColumn<Session>[] = [
    { key: "status", label: "Status", width: "110px" },
    { key: "ipAddress", label: "IP", width: "140px", hideOnMobile: true, formatter: (v) => (v as string) || "—" },
    {
      key: "userAgent",
      label: "User Agent",
      width: "2fr",
      hideOnMobile: true,
      formatter: (v) => truncateUserAgent(v as string | null | undefined)
    },
    { key: "createdAt", label: "Created", width: "160px", formatter: (v) => formatDate(v as string) },
    { key: "lastUsedAt", label: "Last Used", width: "160px", formatter: (v) => formatDate(v as string) }
  ];

  const activityColumns: DataTableColumn<ActivityEntry>[] = [
    {
      key: "occurredAt",
      label: "When",
      width: "180px",
      formatter: (v) => formatDate(v as string)
    },
    {
      key: "action",
      label: "Action",
      width: "140px"
    },
    {
      key: "resourceType",
      label: "Resource",
      width: "140px"
    },
    {
      key: "resourceId",
      label: "Resource ID",
      width: "1.5fr",
      hideOnMobile: true
    },
    {
      key: "actor.email",
      label: "Actor",
      width: "1.5fr",
      formatter: (_v, row) => row.actor?.email ?? "—"
    }
  ];

  function activityActions(row: ActivityEntry): DataTableAction<ActivityEntry>[] {
    return [
      { label: "Copy Activity ID", onClick: () => void copyToClipboard(row.id) },
      { label: "Copy Resource ID", onClick: () => void copyToClipboard(row.resourceId) }
    ];
  }

  function sessionActions(row: Session): DataTableAction<Session>[] {
    const actions: DataTableAction<Session>[] = [
      { label: "Copy Session ID", onClick: () => void copyToClipboard(row.id) }
    ];

    if (row.status === "active") {
      actions.unshift({
        label: "Revoke",
        variant: "danger",
        onClick: () => (sessionToRevoke = row)
      });
    }

    return actions;
  }

  async function handleRevokeSession() {
    if (!user || !sessionToRevoke) return;
    const token = auth.getToken();
    if (!token) return;

    revokingSession = true;
    try {
      await adminCommands.revokeUserSession(user.id, sessionToRevoke.id, fetch, token);
      toastStore.push({ variant: "success", message: "Session revoked" });
      sessionToRevoke = null;
      await pageData.refetch();
    } catch (err) {
      toastStore.push({ variant: "error", message: err instanceof Error ? err.message : "Failed to revoke session" });
    } finally {
      revokingSession = false;
    }
  }

  async function handleSuspend() {
    if (!user) return;
    const token = auth.getToken();
    if (!token) return;

    try {
      await adminCommands.suspendUser(user.id, fetch, token);
      toastStore.push({ variant: "success", message: "User suspended" });
      await pageData.refetch();
    } catch (err) {
      toastStore.push({ variant: "error", message: err instanceof Error ? err.message : "Failed to suspend user" });
    }
  }

  async function handleUnsuspend() {
    if (!user) return;
    const token = auth.getToken();
    if (!token) return;

    try {
      await adminCommands.unsuspendUser(user.id, fetch, token);
      toastStore.push({ variant: "success", message: "User reactivated" });
      await pageData.refetch();
    } catch (err) {
      toastStore.push({ variant: "error", message: err instanceof Error ? err.message : "Failed to reactivate user" });
    }
  }

  async function handleRoleChange() {
    if (!user) return;
    const token = auth.getToken();
    if (!token) return;

    updatingRole = true;
    try {
      await adminCommands.updateUserRole(user.id, { role: selectedRole }, fetch, token);
      toastStore.push({ variant: "success", message: "Role updated" });
      showRoleDialog = false;
      await pageData.refetch();
    } catch (err) {
      toastStore.push({ variant: "error", message: err instanceof Error ? err.message : "Failed to update role" });
    } finally {
      updatingRole = false;
    }
  }

  const roleItems = [
    { value: UserRoleConst.User, label: "User" },
    { value: UserRoleConst.Tester, label: "Tester" },
    { value: UserRoleConst.Editor, label: "Editor" },
    { value: UserRoleConst.Admin, label: "Admin" },
    { value: UserRoleConst.Support, label: "Support" },
    { value: UserRoleConst.Superadmin, label: "Superadmin" }
  ];

  const userMenuItems = $derived(() => {
    if (!user) return [];

    return [
      {
        label: "Edit",
        onSelect: () =>
          void gotoWithContext(`/users/${user.id}/edit`, {
            label: "User",
            href: `/users/${user.id}`,
            type: "detail"
          })
      },
      {
        label: "Copy ID",
        onSelect: () => void copyToClipboard(user.id)
      },
      {
        label: "Copy Email",
        onSelect: () => void copyToClipboard(user.email)
      },
      { separator: true },
      {
        label: "Change role…",
        onSelect: () => {
          showRoleDialog = true;
        }
      },
      user.status === "active"
        ? {
            label: "Suspend user",
            destructive: true,
            onSelect: () => void handleSuspend()
          }
        : user.status === "suspended"
          ? {
              label: "Reactivate user",
              onSelect: () => void handleUnsuspend()
            }
          : {
              label: "User deleted",
              disabled: true
            }
    ];
  });
</script>

{#if pageData.loading}
  <PageLoading message="Loading user..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if user}
  <PageHeader
    title={user.email}
    subtitle={user.displayName ?? undefined}
    backHref={computedBackInfo.href}
    backLabel={computedBackInfo.label}
    backIsContextual={computedBackInfo.isContextual ?? false}
    bannerMessage={user.status !== "active" ? `User status: ${user.status}` : undefined}
  >
    {#snippet actions()}
      <DropdownMenu items={userMenuItems}>
        {#snippet trigger()}
          <MoreVertical size={16} aria-hidden="true" />
        {/snippet}
      </DropdownMenu>
    {/snippet}

    <PageHeaderMeta>
      <PageHeaderMetaRow>
        <PageHeaderMetaItem label="ID">
          <Code copy>{user.id}</Code>
        </PageHeaderMetaItem>
        <PageHeaderMetaSeparator />
        <Pill accent={getRoleAccent(user.role)}>{user.role}</Pill>
        <Pill accent={getStatusAccent(user.status)}>{user.status}</Pill>
      </PageHeaderMetaRow>
    </PageHeaderMeta>
  </PageHeader>

  <div class="user-view">
    <TabsRoot bind:value={activeTab} variant="boxed" size="sm" historyKey="tab">
      <TabsList>
        <TabsTrigger value="details">Details</TabsTrigger>
        <TabsTrigger value="sessions" count={sessions.length}>Sessions</TabsTrigger>
        <TabsTrigger value="activity" count={activity.length}>Activity</TabsTrigger>
      </TabsList>

      <TabsContent value="details">
        <DetailsCard>
          <DetailsSection legend="Account">
            <DetailsItem label="Created" value={formatDate(user.createdAt)} />
            <DetailsItem label="Updated" value={formatDate(user.updatedAt)} />
          </DetailsSection>

          <DetailsSection legend="Security">
            <DetailsItem label="Active sessions" value={user.activeSessionCount} />
            <DetailsItem label="Failed logins" value={user.failedLoginCount} />
            <DetailsItem label="Lockout until" value={user.lockoutUntil ? formatDate(user.lockoutUntil) : null} />
          </DetailsSection>
        </DetailsCard>
      </TabsContent>

      <TabsContent value="sessions">
        <DataTable
          data={sessions}
          columns={sessionColumns}
          actions={sessionActions}
          emptyMessage="No sessions found"
          showLimitSelector={false}
        >
          {#snippet cell({ column, value })}
            {#if column.key === "status"}
              <Pill accent={getSessionStatusAccent(value)}>{value}</Pill>
            {:else if column.key === "ipAddress"}
              <code>{value || "—"}</code>
            {:else}
              {value}
            {/if}
          {/snippet}
        </DataTable>
      </TabsContent>

      <TabsContent value="activity">
        <DataTable
          data={activity}
          columns={activityColumns}
          actions={activityActions}
          emptyMessage="No activity recorded for this user"
          showLimitSelector={false}
        >
          {#snippet cell({ column, value })}
            {#if column.key === "action"}
              <Pill accent={getActivityAccent(value)}>{value}</Pill>
            {:else if column.key === "resourceId"}
              <code>{value}</code>
            {:else}
              {value}
            {/if}
          {/snippet}
        </DataTable>
      </TabsContent>
    </TabsRoot>
  </div>
{/if}

<Dialog
  bind:open={showRoleDialog}
  title="Change role"
  description="Select a new role for this user."
  showTrigger={false}
>
  <Field label="Role">
    <Select
      value={selectedRole}
      onchange={(v) => { selectedRole = v as UserRole; }}
      items={roleItems}
      placeholder="Select role"
    />
  </Field>

  {#snippet footer()}
    <div class="user-view__dialog-footer">
      <Button type="button" variant="secondary" onclick={() => (showRoleDialog = false)} disabled={updatingRole}>
        Cancel
      </Button>
      <Button type="button" variant="primary" onclick={() => void handleRoleChange()} disabled={updatingRole}>
        Save
      </Button>
    </div>
  {/snippet}
</Dialog>

<AlertDialog
  bind:open={showRevokeDialog}
  title="Revoke session"
  description="This will immediately log the user out of this session. They will need to log in again."
  confirmLabel={revokingSession ? "Revoking..." : "Revoke"}
  showTrigger={false}
  onConfirm={handleRevokeSession}
  onCancel={() => (sessionToRevoke = null)}
/>

<style>
  .user-view {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .user-view__dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
  }
</style>
