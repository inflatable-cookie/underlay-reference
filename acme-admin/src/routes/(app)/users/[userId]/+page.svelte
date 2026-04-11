<script lang="ts">
import {
  useToasts
} from "@decodelabs/underlay/runtime/feedback";
import {
  useAuthenticatedData
} from "@decodelabs/underlay/runtime/auth";
import {
  computeBackInfo,
  consumeNavigationContext
} from "@decodelabs/underlay/runtime/navigation";
import {
  AlertDialog as PoodleAlertDialog,
  Callout as PoodleCallout,
  Card as PoodleCard,
  Code as PoodleCode,
  DetailItem as PoodleDetailItem,
  formatDisplayDateTime,
  MetaBar as PoodleMetaBar,
  MetaItem as PoodleMetaItem,
  Tabs
  } from "@poodle/svelte";
  import { DetailSection as PoodleDetailSection,
  FormDialog,
  PageHeader as PoodlePageHeader,
  PageLoading } from "@poodle/svelte";
  import { goto } from "$app/navigation";
  import {
    Button as PoodleButton,
  Field as PoodleField,
  FormActions as PoodleFormActions,
  IconButton as PoodleIconButton,
  Menu as PoodleMenu,
  Pill as PoodlePill,
  Select as PoodleSelect,
  type MenuItem
  } from "@poodle/svelte";
  import { DataTable,
  type TableColumn,
  type TableRow,
  type TableRowAction } from "@poodle/svelte";
    import { gotoWithContext } from "@decodelabs/underlay/client/navigation";
  import {
    adminCommands,
    type ActivityEntry,
    type Session,
    type UserDetail,
    type UserRole,
    UserRole as UserRoleConst
  } from "@api-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import {
    getUserRoleTone,
    getUserStatusTone,
    getSessionStatusTone,
    getActivityTone
  } from "$lib/utils/accents";
  interface Props {
    data: { userId: string };
  }

  let { data }: Props = $props();
  const toastStore = useToasts();

  const defaultBackHref = "/users";
  const { backInfo } = consumeNavigationContext("Back to users", defaultBackHref);

  const userData = useAuthenticatedData(
    async (fetch, token) => {
      return adminCommands.getUser(data.userId, fetch, token);
    },
    {
      defaultValue: null as UserDetail | null
    }
  );

  const sessionsData = useAuthenticatedData(
    async (fetch, token) => adminCommands.listUserSessions(data.userId, fetch, token),
    {
      getAuthLoading: () => true,
      defaultValue: [] as Session[]
    }
  );

  const activityData = useAuthenticatedData(
    async (fetch, token) => {
      const activity = await adminCommands.listActivityForUser(data.userId, fetch, token, { limit: 10 });
      return activity.data;
    },
    {
      getAuthLoading: () => true,
      defaultValue: [] as ActivityEntry[]
    }
  );

  let activeTab = $state("details");
  const mountedTabsSet = new Set<string>();
  let mountedTabsVersion = $state(0);

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

  $effect(() => {
    if (activeTab && !mountedTabsSet.has(activeTab)) {
      mountedTabsSet.add(activeTab);
      mountedTabsVersion++;
    }
  });

  function isTabMounted(value: string): boolean {
    void mountedTabsVersion;
    return mountedTabsSet.has(value);
  }

  const user = $derived(userData.data ?? null);
  const sessions = $derived(sessionsData.data ?? []);
  const activity = $derived(activityData.data ?? []);
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

  const sessionColumns: TableColumn[] = [
    { id: "status", label: "Status", width: "110px" },
    { id: "ipAddress", label: "IP", width: "140px", hideOnMobile: true },
    {
      id: "userAgent",
      label: "User Agent",
      width: "2fr",
      hideOnMobile: true
    },
    { id: "createdAt", label: "Created", width: "160px" },
    { id: "lastUsedAt", label: "Last Used", width: "160px" }
  ];

  const activityColumns: TableColumn[] = [
    {
      id: "occurredAt",
      label: "When",
      width: "180px"
    },
    {
      id: "action",
      label: "Action",
      width: "140px"
    },
    {
      id: "resourceType",
      label: "Resource",
      width: "140px"
    },
    {
      id: "resourceId",
      label: "Resource ID",
      width: "1.5fr"
    },
    {
      id: "actor.email",
      label: "Actor",
      width: "1.5fr"
    }
  ];

  function activityActions(_row: TableRow): TableRowAction[] {
    return [
      { value: "copy-activity-id", label: "Copy Activity ID" },
      { value: "copy-resource-id", label: "Copy Resource ID" }
    ];
  }

  function sessionActions(row: TableRow): TableRowAction[] {
    const session = row.data as Session | undefined;
    if (!session) {
      return [];
    }

    const actions: TableRowAction[] = [
      { value: "copy-session-id", label: "Copy Session ID" }
    ];

    if (session.status === "active") {
      actions.unshift({
        value: "revoke",
        label: "Revoke",
        tone: "danger"
      });
    }

    return actions;
  }

  function handleSessionRowAction(event: CustomEvent<{ rowId: string; row: TableRow; action: TableRowAction }>) {
    const session = event.detail.row.data as Session | undefined;
    if (!session) {
      return;
    }

    switch (event.detail.action.value) {
      case "revoke":
        sessionToRevoke = session;
        break;
      case "copy-session-id":
        void copyToClipboard(session.id);
        break;
    }
  }

  function handleActivityRowAction(event: CustomEvent<{ rowId: string; row: TableRow; action: TableRowAction }>) {
    const entry = event.detail.row.data as ActivityEntry | undefined;
    if (!entry) {
      return;
    }

    switch (event.detail.action.value) {
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
      await Promise.all([userData.refetch(), sessionsData.refetch()]);
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
      await userData.refetch();
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
      await userData.refetch();
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
      await userData.refetch();
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
</script>

{#if userData.loading}
  <PageLoading presentation="inline" message="Loading user..." />
{:else if userData.error}
  <PoodleCallout tone="danger" message={userData.error} announceMode="polite" />
{:else if user}
  <section class="user-view">
    <div class="user-view__header">
      <PoodlePageHeader
        section="User"
        title={user.email}
        subtitle={user.displayName ?? undefined}
        backHref={computedBackInfo.href}
        backLabel={computedBackInfo.label}
        backIsContextual={computedBackInfo.isContextual ?? false}
        bannerMessage={user.status !== "active" ? `User status: ${user.status}` : undefined}
      >
        {#snippet actions()}
          <PoodleMenu items={getUserMenuItems(user)} ariaLabel="User actions" placement="bottom-end" on:action={(event) => handleUserMenuAction(user, event.detail.value)}>
            <PoodleIconButton slot="trigger" icon="ellipsis" ariaLabel="User actions" />
          </PoodleMenu>
        {/snippet}
      </PoodlePageHeader>

      <div class="user-view__meta">
      <PoodleMetaBar ariaLabel="User metadata">
        <PoodleMetaItem label="ID">
          <PoodleCode inline source={user.id} showCopyButton />
        </PoodleMetaItem>
        <PoodlePill tone={getUserRoleTone(user.role)} appearance="badge" size="lg">{user.role}</PoodlePill>
        <PoodlePill tone={getUserStatusTone(user.status)} appearance="badge" size="lg">{user.status}</PoodlePill>
      </PoodleMetaBar>
      </div>
    </div>

    <Tabs
      bind:value={activeTab}
      items={[
        { value: "details", label: "Details" },
        { value: "sessions", label: "Sessions", count: user.activeSessionCount },
        { value: "activity", label: "Activity" }
      ]}
      variant="card"
      size="sm"
      ariaLabel="Detail sections"
      let:activeValue
    >
      {#if isTabMounted(activeValue)}
      {#if activeValue === "details"}
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
      {:else if activeValue === "sessions"}
        {#if activeTab === "sessions" && sessionsData.loading}
          <PageLoading presentation="inline" message="Loading sessions..." />
        {:else if sessionsData.error}
          <PoodleCallout tone="danger" message={sessionsData.error} announceMode="polite" />
        {:else}
          <DataTable
            rows={sessionRows}
            columns={sessionColumns}
            rowActions={sessionActions}
            emptyMessage="No sessions found"
            showLimitSelector={false}
            on:rowActionSelect={handleSessionRowAction}
          >
            <svelte:fragment slot="cell" let:column let:value>
              {#if column.id === "status"}
                <PoodlePill tone={getSessionStatusTone(String(value ?? ""))} appearance="badge" size="lg">{value}</PoodlePill>
              {:else if column.id === "ipAddress"}
                <code>{value || "—"}</code>
              {:else}
                {value}
              {/if}
            </svelte:fragment>
          </DataTable>
        {/if}
      {:else if activeValue === "activity"}
        {#if activeTab === "activity" && activityData.loading}
          <PageLoading presentation="inline" message="Loading activity..." />
        {:else if activityData.error}
          <PoodleCallout tone="danger" message={activityData.error} announceMode="polite" />
        {:else}
          <DataTable
            rows={activityRows}
            columns={activityColumns}
            rowActions={activityActions}
            emptyMessage="No activity recorded for this user"
            showLimitSelector={false}
            on:rowActionSelect={handleActivityRowAction}
          >
            <svelte:fragment slot="cell" let:column let:value>
              {#if column.id === "action"}
                <PoodlePill tone={getActivityTone(String(value ?? ""))} appearance="badge" size="lg">{value}</PoodlePill>
              {:else if column.id === "resourceId"}
                <code>{value}</code>
              {:else}
                {value}
              {/if}
            </svelte:fragment>
          </DataTable>
        {/if}
      {/if}
      {/if}
    </Tabs>
  </section>
{/if}

<FormDialog
  bind:open={showRoleDialog}
  title="Change role"
  subtitle="Select a new role for this user."
  submitting={updatingRole}
  showDefaultActions={false}
  on:cancel={() => (showRoleDialog = false)}
>
  <form
      id="user-role-form"
      onsubmit={(event) => {
        event.preventDefault();
        void handleRoleChange();
      }}
    >
      <PoodleField id="user-role-dialog" label="Role" let:describedBy>
        <PoodleSelect
          id="user-role-dialog"
          value={selectedRole}
          describedBy={describedBy}
          options={roleItems}
          placeholder="Select role"
          disabled={updatingRole}
          on:valueChange={(event) => { selectedRole = event.detail.value as UserRole; }}
        />
      </PoodleField>
  </form>
  <svelte:fragment slot="actions">
    <PoodleFormActions align="end">
      <PoodleButton type="button" variant="ghost" disabled={updatingRole} on:click={() => (showRoleDialog = false)}>
        Cancel
      </PoodleButton>
      <PoodleButton type="submit" form="user-role-form" variant="primary" disabled={updatingRole}>
        {updatingRole ? "Saving..." : "Save"}
      </PoodleButton>
    </PoodleFormActions>
  </svelte:fragment>
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

<style>
  .user-view {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .user-view__header {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .user-view__meta {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
</style>
