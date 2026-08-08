<script lang="ts">
  import { gotoWithContext } from "@inflatable-cookie/underlay/client/navigation";
  import { EntityDetailPage } from "@inflatable-cookie/underlay/templates";
  import { useAuthenticatedData } from "@inflatable-cookie/underlay/runtime/auth";
  import { useToasts } from "@inflatable-cookie/underlay/runtime/feedback";
  import {
    computeBackInfo,
    consumeNavigationContext
  } from "@inflatable-cookie/underlay/runtime/navigation";
  import {
    AlertDialog as PoodleAlertDialog,
    Button as PoodleButton,
    Callout as PoodleCallout,
    Card as PoodleCard,
    Code as PoodleCode,
    DataTable,
    DetailItem as PoodleDetailItem,
    DetailSection as PoodleDetailSection,
    Field as PoodleField,
    FormActions as PoodleFormActions,
    FormDialog,
    IconButton as PoodleIconButton,
    Menu as PoodleMenu,
    Pill as PoodlePill,
    Select as PoodleSelect,
    type MenuItem,
    type TableColumn,
    type TableRow,
    type TableRowAction,
    formatDisplayDateTime
  } from "@inflatable-cookie/poodle-svelte";
  import {
    adminCommands,
    type ActivityEntry,
    type ActivityListResponse,
    type PagedListResponse,
    type Session,
    type UserDetail,
    type UserRole,
    UserRole as UserRoleConst
  } from "@api-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import {
    getActivityTone,
    getSessionStatusTone,
    getUserRoleTone,
    getUserStatusTone
  } from "$lib/utils/accents";

  interface Props {
    data: { userId: string };
  }

  let { data }: Props = $props();

  const toastStore = useToasts();
  const defaultBackHref = "/users";
  const { backInfo } = consumeNavigationContext("Back to users", defaultBackHref);

  let user = $state<UserDetail | null>(null);
  let reloadRevision = $state(0);
  let activeTab = $state("details");

  async function userLoader(fetch: typeof window.fetch, token: string | null) {
    if (!token) throw new Error("Not authenticated");
    const result = await adminCommands.getUser(data.userId, fetch, token);
    user = result;
    return result;
  }

  const sessionsData = useAuthenticatedData(
    async (fetch, token) => adminCommands.listUserSessions(data.userId, fetch, token),
    {
      getAuthLoading: () => true,
      defaultValue: {
        data: [],
        total: 0,
        hasMore: false
      } as PagedListResponse<Session>
    }
  );

  const activityData = useAuthenticatedData(
    async (fetch, token) =>
      adminCommands.listActivityForUser(data.userId, fetch, token, { limit: 10 }),
    {
      getAuthLoading: () => true,
      defaultValue: {
        data: [],
        total: 0,
        hasMore: false
      } as ActivityListResponse
    }
  );

  $effect(() => {
    if (activeTab === "sessions") {
      sessionsData.tryFetch($authLoading, $currentUser);
    }
  });

  $effect(() => {
    if (activeTab === "activity") {
      activityData.tryFetch($authLoading, $currentUser);
    }
  });

  const sessions = $derived(sessionsData.data?.data ?? []);
  const activity = $derived(activityData.data?.data ?? []);

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

  const sessionRows = $derived<TableRow<Session>[]>(
    sessions.map((session) => ({
      id: session.id,
      cells: {
        status: session.status,
        ipAddress: session.ipAddress || "—",
        userAgent: truncateUserAgent(session.userAgent),
        createdAt: formatDisplayDateTime(session.createdAt),
        lastUsedAt: formatDisplayDateTime(session.lastUsedAt)
      },
      data: session
    }))
  );

  const activityRows = $derived<TableRow<ActivityEntry>[]>(
    activity.map((entry) => ({
      id: entry.id,
      cells: {
        occurredAt: formatDisplayDateTime(entry.occurredAt),
        action: entry.action,
        resourceType: entry.resourceType,
        resourceId: entry.resourceId,
        "actor.email": entry.actor?.email ?? "—"
      },
      data: entry
    }))
  );

  let sessionToRevoke = $state<Session | null>(null);
  let showRevokeDialog = $state(false);
  let revokingSession = $state(false);
  let showRoleDialog = $state(false);
  let selectedRole = $state<UserRole>(UserRoleConst.User);
  let updatingRole = $state(false);

  $effect(() => {
    showRevokeDialog = sessionToRevoke !== null;
  });

  $effect(() => {
    if (user) {
      selectedRole = user.role;
    }
  });

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
      // Fall through to legacy approach.
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

  const sessionColumns: TableColumn[] = [
    { id: "status", label: "Status", width: "110px" },
    { id: "ipAddress", label: "IP", width: "140px", hideOnMobile: true },
    { id: "userAgent", label: "User Agent", width: "2fr", hideOnMobile: true },
    { id: "createdAt", label: "Created", width: "160px" },
    { id: "lastUsedAt", label: "Last Used", width: "160px" }
  ];

  const activityColumns: TableColumn[] = [
    { id: "occurredAt", label: "When", width: "180px" },
    { id: "action", label: "Action", width: "140px" },
    { id: "resourceType", label: "Resource", width: "140px" },
    { id: "resourceId", label: "Resource ID", width: "1.5fr" },
    { id: "actor.email", label: "Actor", width: "1.5fr" }
  ];

  function activityActions(_row: TableRow): TableRowAction[] {
    return [
      { value: "copy-activity-id", label: "Copy Activity ID" },
      { value: "copy-resource-id", label: "Copy Resource ID" }
    ];
  }

  function sessionActions(row: TableRow): TableRowAction[] {
    const session = row.data as Session | undefined;
    if (!session) return [];

    const actions: TableRowAction[] = [
      { value: "copy-session-id", label: "Copy Session ID" }
    ];

    if (session.status === "active") {
      actions.unshift({ value: "revoke", label: "Revoke", tone: "danger" });
    }

    return actions;
  }

  function handleSessionRowAction(payload: { rowId: string; row: TableRow; action: TableRowAction }) {
    const session = payload.row.data as Session | undefined;
    if (!session) return;

    switch (payload.action.value) {
      case "revoke":
        sessionToRevoke = session;
        break;
      case "copy-session-id":
        void copyToClipboard(session.id);
        break;
    }
  }

  function handleActivityRowAction(payload: { rowId: string; row: TableRow; action: TableRowAction }) {
    const entry = payload.row.data as ActivityEntry | undefined;
    if (!entry) return;

    switch (payload.action.value) {
      case "copy-activity-id":
        void copyToClipboard(entry.id);
        break;
      case "copy-resource-id":
        void copyToClipboard(entry.resourceId);
        break;
    }
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
      reloadRevision += 1;
      await sessionsData.refetch();
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to revoke session";
      toastStore.push({ variant: "error", message });
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
      reloadRevision += 1;
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to suspend user";
      toastStore.push({ variant: "error", message });
    }
  }

  async function handleUnsuspend() {
    if (!user) return;
    const token = auth.getToken();
    if (!token) return;

    try {
      await adminCommands.unsuspendUser(user.id, fetch, token);
      toastStore.push({ variant: "success", message: "User reactivated" });
      reloadRevision += 1;
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to reactivate user";
      toastStore.push({ variant: "error", message });
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
      reloadRevision += 1;
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to update role";
      toastStore.push({ variant: "error", message });
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

  function getUserMenuItems(currentUser: UserDetail): MenuItem[] {
    const items: MenuItem[] = [
      { value: "edit", label: "Edit" },
      { value: "copy-id", label: "Copy ID" },
      { value: "copy-email", label: "Copy Email" },
      { value: "separator", label: "", kind: "separator" },
      { value: "change-role", label: "Change role…" }
    ];

    if (currentUser.status === "active") {
      items.push({ value: "suspend", label: "Suspend user" });
    } else if (currentUser.status === "suspended") {
      items.push({ value: "reactivate", label: "Reactivate user" });
    } else {
      items.push({ value: "deleted", label: "User deleted", disabled: true });
    }

    return items;
  }

  function handleUserMenuAction(currentUser: UserDetail, value: string) {
    if (value === "edit") {
      void gotoWithContext(`/users/${currentUser.id}/edit`, {
        label: "User",
        href: `/users/${currentUser.id}`,
        type: "detail"
      });
      return;
    }

    if (value === "copy-id") {
      void copyToClipboard(currentUser.id);
      return;
    }

    if (value === "copy-email") {
      void copyToClipboard(currentUser.email);
      return;
    }

    if (value === "change-role") {
      showRoleDialog = true;
      return;
    }

    if (value === "suspend") {
      void handleSuspend();
      return;
    }

    if (value === "reactivate") {
      void handleUnsuspend();
    }
  }

  const headerMeta = $derived.by(() => [
    { label: "ID", value: idSnippet },
    { label: "", value: roleSnippet, separator: false },
    { label: "", value: statusSnippet, separator: false }
  ]);

  const detailTabs = $derived.by(() => [
    { id: "details", label: "Details", content: detailsTabSnippet },
    { id: "sessions", label: "Sessions", count: user?.activeSessionCount, content: sessionsTabSnippet },
    { id: "activity", label: "Activity", content: activityTabSnippet }
  ]);
</script>

<EntityDetailPage
  title={user?.displayName?.trim() || user?.email || "User"}
  section="Users"
  subtitle={user?.displayName?.trim() && user.displayName.trim() !== user.email ? user.email : undefined}
  backHref={computedBackInfo.href}
  backLabel={computedBackInfo.label}
  bannerMessage={user && user.status !== "active" ? `User status: ${user.status}` : undefined}
  dataLoader={userLoader}
  reloadKey={reloadRevision}
  meta={headerMeta}
  headerActions={headerActionsSnippet}
  tabs={detailTabs}
  tabsSize="sm"
  keepMountedTabs
  onTabChange={(tabId) => {
    activeTab = tabId;
  }}
/>

{#snippet idSnippet()}
  {#if user}
    <PoodleCode inline inlineVariant="plain" typography="inline" source={user.id} showCopyButton />
  {/if}
{/snippet}

{#snippet roleSnippet()}
  {#if user}
    <PoodlePill tone={getUserRoleTone(user.role)} appearance="badge" size="sm" typography="inherit">{user.role}</PoodlePill>
  {/if}
{/snippet}

{#snippet statusSnippet()}
  {#if user}
    <PoodlePill tone={getUserStatusTone(user.status)} appearance="badge" size="sm" typography="inherit">{user.status}</PoodlePill>
  {/if}
{/snippet}

{#snippet headerActionsSnippet()}
  {#if user}
    {@const currentDetailUser = user}
    <PoodleMenu items={getUserMenuItems(currentDetailUser)} ariaLabel="User actions" placement="bottom-end" onAction={(value) => handleUserMenuAction(currentDetailUser, value)}>
      {#snippet trigger()}
        <PoodleIconButton icon="ellipsis" ariaLabel="User actions" />
      {/snippet}
    </PoodleMenu>
  {/if}
{/snippet}

{#snippet detailsTabSnippet()}
  {#if user}
    <PoodleCard>
      <PoodleDetailSection title="Account" columns={2} separated={false}>
        <PoodleDetailItem presentation="surface" label="Created" value={formatDisplayDateTime(user.createdAt)} />
        <PoodleDetailItem presentation="surface" label="Updated" value={formatDisplayDateTime(user.updatedAt)} />
      </PoodleDetailSection>

      <PoodleDetailSection title="Security" columns={2} separated={false}>
        <PoodleDetailItem presentation="surface" label="Active sessions" value={String(user.activeSessionCount)} />
        <PoodleDetailItem presentation="surface" label="Failed logins" value={String(user.failedLoginCount)} />
        <PoodleDetailItem presentation="surface" label="Lockout until" value={user.lockoutUntil ? formatDisplayDateTime(user.lockoutUntil) : "—"} />
      </PoodleDetailSection>
    </PoodleCard>
  {/if}
{/snippet}

{#snippet sessionsTabSnippet()}
  {#if activeTab === "sessions" && sessionsData.loading}
    <PoodleCallout tone="info" message="Loading sessions..." announceMode="polite" />
  {:else if sessionsData.error}
    <PoodleCallout tone="danger" message={sessionsData.error} announceMode="polite" />
  {:else}
    <DataTable
      rows={sessionRows}
      columns={sessionColumns}
      rowActions={sessionActions}
      emptyMessage="No sessions found"
      showLimitSelector={false}
      onRowActionSelect={handleSessionRowAction}
    >
      {#snippet cell(column, row, value)}
        {#if column.id === "status"}
          <PoodlePill tone={getSessionStatusTone(String(value ?? ""))} appearance="badge" size="lg">{value}</PoodlePill>
        {:else if column.id === "ipAddress"}
          <code>{value || "—"}</code>
        {:else}
          {value}
        {/if}
      {/snippet}
    </DataTable>
  {/if}
{/snippet}

{#snippet activityTabSnippet()}
  {#if activeTab === "activity" && activityData.loading}
    <PoodleCallout tone="info" message="Loading activity..." announceMode="polite" />
  {:else if activityData.error}
    <PoodleCallout tone="danger" message={activityData.error} announceMode="polite" />
  {:else}
    <DataTable
      rows={activityRows}
      columns={activityColumns}
      rowActions={activityActions}
      emptyMessage="No activity recorded for this user"
      showLimitSelector={false}
      onRowActionSelect={handleActivityRowAction}
    >
      {#snippet cell(column, row, value)}
        {#if column.id === "action"}
          <PoodlePill tone={getActivityTone(String(value ?? ""))} appearance="badge" size="lg">{value}</PoodlePill>
        {:else if column.id === "resourceId"}
          <code>{value}</code>
        {:else}
          {value}
        {/if}
      {/snippet}
    </DataTable>
  {/if}
{/snippet}

<FormDialog
  bind:open={showRoleDialog}
  title="Change role"
  subtitle="Select a new role for this user."
  submitting={updatingRole}
  showDefaultActions={false}
  onCancel={() => (showRoleDialog = false)}
>
  <form
    id="user-role-form"
    onsubmit={(event) => {
      event.preventDefault();
      void handleRoleChange();
    }}
  >
    <PoodleField id="user-role-dialog" label="Role">
      {#snippet control({ describedBy })}
        <PoodleSelect
          id="user-role-dialog"
          value={selectedRole}
          describedBy={describedBy}
          options={roleItems}
          placeholder="Select role"
          disabled={updatingRole}
          onValueChange={(value) => {
            selectedRole = value as UserRole;
          }}
        />
      {/snippet}
    </PoodleField>
  </form>
  {#snippet actions(submitting)}
    <PoodleFormActions align="end" showTopBorder>
      <PoodleButton type="button" variant="ghost" disabled={updatingRole} onClick={() => (showRoleDialog = false)}>
        Cancel
      </PoodleButton>
      <PoodleButton type="submit" form="user-role-form" variant="primary" disabled={updatingRole}>
        {updatingRole ? "Saving..." : "Save"}
      </PoodleButton>
    </PoodleFormActions>
  {/snippet}
</FormDialog>

<PoodleAlertDialog
  bind:open={showRevokeDialog}
  title="Revoke session"
  description="This will immediately log the user out of this session. They will need to log in again."
  confirmLabel={revokingSession ? "Revoking..." : "Revoke"}
  onConfirm={handleRevokeSession}
  onCancel={() => (sessionToRevoke = null)}
  tone="danger"
/>
