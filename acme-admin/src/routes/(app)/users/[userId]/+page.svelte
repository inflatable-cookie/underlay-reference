<script lang="ts">
  import {
    AlertDialog as PoodleAlertDialog,
    Callout as PoodleCallout
  } from "@poodle/svelte-primitives";
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
  } from "@poodle/svelte-primitives";
  import {
    DataTable,
    DetailsCard,
    DetailsItem,
    DetailsSection,
        PageLoading,
    type DataTableAction,
    type DataTableColumn
  } from "@decodelabs/underlay/components";
  import {
    FormDialog,
    DetailPageShell,
    DetailMeta,
    DetailMetaId,
    DetailMetaSeparator,
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

  const user = $derived(userData.data ?? null);
  const sessions = $derived(sessionsData.data ?? []);
  const activity = $derived(activityData.data ?? []);

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

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleString();
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
  <PageLoading message="Loading user..." />
{:else if userData.error}
  <PoodleCallout tone="danger" message={userData.error} announceMode="polite" />
{:else if user}
  <DetailPageShell
    section="User"
    title={user.email}
    subtitle={user.displayName ?? undefined}
    backHref={computedBackInfo.href}
    backLabel={computedBackInfo.label}
    backIsContextual={computedBackInfo.isContextual ?? false}
    bannerMessage={user.status !== "active" ? `User status: ${user.status}` : undefined}
    tabs={[
      { value: "details", label: "Details" },
      { value: "sessions", label: "Sessions", count: user.activeSessionCount },
      { value: "activity", label: "Activity" }
    ]}
    bind:activeTab
  >
    {#snippet meta()}
      <DetailMeta>
        <DetailMetaId value={user.id} />
        <DetailMetaSeparator />
        <PoodlePill tone={getUserRoleTone(user.role)} appearance="badge" size="lg">{user.role}</PoodlePill>
        <PoodlePill tone={getUserStatusTone(user.status)} appearance="badge" size="lg">{user.status}</PoodlePill>
      </DetailMeta>
    {/snippet}

    {#snippet actions()}
      <PoodleMenu items={getUserMenuItems(user)} ariaLabel="User actions" placement="bottom-end" on:action={(event) => handleUserMenuAction(user, event.detail.value)}>
        <PoodleIconButton slot="trigger" icon="ellipsis" ariaLabel="User actions" />
      </PoodleMenu>
    {/snippet}

    {#snippet tabContent(tab)}
      {#if tab === "details"}
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
      {:else if tab === "sessions"}
        {#if activeTab === "sessions" && sessionsData.loading}
          <PageLoading message="Loading sessions..." />
        {:else if sessionsData.error}
          <PoodleCallout tone="danger" message={sessionsData.error} announceMode="polite" />
        {:else}
          <DataTable
            data={sessions}
            columns={sessionColumns}
            actions={sessionActions}
            emptyMessage="No sessions found"
            showLimitSelector={false}
          >
            {#snippet cell({ column, value })}
              {#if column.key === "status"}
                <PoodlePill tone={getSessionStatusTone(value)} appearance="badge" size="lg">{value}</PoodlePill>
              {:else if column.key === "ipAddress"}
                <code>{value || "—"}</code>
              {:else}
                {value}
              {/if}
            {/snippet}
          </DataTable>
        {/if}
      {:else if tab === "activity"}
        {#if activeTab === "activity" && activityData.loading}
          <PageLoading message="Loading activity..." />
        {:else if activityData.error}
          <PoodleCallout tone="danger" message={activityData.error} announceMode="polite" />
        {:else}
          <DataTable
            data={activity}
            columns={activityColumns}
            actions={activityActions}
            emptyMessage="No activity recorded for this user"
            showLimitSelector={false}
          >
            {#snippet cell({ column, value })}
              {#if column.key === "action"}
                <PoodlePill tone={getActivityTone(value)} appearance="badge" size="lg">{value}</PoodlePill>
              {:else if column.key === "resourceId"}
                <code>{value}</code>
              {:else}
                {value}
              {/if}
            {/snippet}
          </DataTable>
        {/if}
      {/if}
    {/snippet}
  </DetailPageShell>
{/if}

<FormDialog
  bind:open={showRoleDialog}
  title="Change role"
  subtitle="Select a new role for this user."
  submitting={updatingRole}
  onCancel={() => (showRoleDialog = false)}
>
  {#snippet children(submitting)}
    <form
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
          disabled={submitting}
          on:valueChange={(event) => { selectedRole = event.detail.value as UserRole; }}
        />
      </PoodleField>

      <PoodleFormActions align="end">
        <PoodleButton type="button" variant="ghost" disabled={submitting} on:click={() => (showRoleDialog = false)}>
          Cancel
        </PoodleButton>
        <PoodleButton type="submit" variant="primary" disabled={submitting}>
          {submitting ? "Saving..." : "Save"}
        </PoodleButton>
      </PoodleFormActions>
    </form>
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
