<script lang="ts">
  import { goto } from "$app/navigation";
  import {
    AlertDialog,
    Button,
    DataTable,
    DetailsCard,
    DetailsItem,
    DetailsSection,
    DropdownMenu,
    Field,
    FormActions,
    FormError,
    PageLoading,
    Pill,
    Select,
    TextButton,
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
  import { getUserRoleAccent, getUserStatusAccent, getSessionStatusAccent, getActivityAccent } from "$lib/utils/accents";
  import MoreVertical from "lucide-svelte/icons/more-vertical";

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
      getToken: () => auth.getToken(),
      defaultValue: null as UserDetail | null
    }
  );

  const sessionsData = useAuthenticatedData(
    async (fetch, token) => adminCommands.listUserSessions(data.userId, fetch, token),
    {
      getToken: () => auth.getToken(),
      defaultValue: [] as Session[]
    }
  );

  const activityData = useAuthenticatedData(
    async (fetch, token) => {
      const activity = await adminCommands.listActivityForUser(data.userId, fetch, token, { limit: 10 });
      return activity.data;
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: [] as ActivityEntry[]
    }
  );

  let activeTab = $state("details");

  $effect(() => {
    userData.tryFetch($authLoading, $currentUser);
  });

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

  function getUserMenuItems(currentUser: UserDetail) {
    return [
      {
        label: "Edit",
        onSelect: () =>
          void gotoWithContext(`/users/${currentUser.id}/edit`, {
            label: "User",
            href: `/users/${currentUser.id}`,
            type: "detail"
          })
      },
      {
        label: "Copy ID",
        onSelect: () => void copyToClipboard(currentUser.id)
      },
      {
        label: "Copy Email",
        onSelect: () => void copyToClipboard(currentUser.email)
      },
      { separator: true },
      {
        label: "Change role…",
        onSelect: () => {
          showRoleDialog = true;
        }
      },
      currentUser.status === "active"
        ? {
            label: "Suspend user",
            destructive: true,
            onSelect: () => void handleSuspend()
          }
        : currentUser.status === "suspended"
          ? {
              label: "Reactivate user",
              onSelect: () => void handleUnsuspend()
            }
          : {
              label: "User deleted",
              disabled: true
            }
    ];
  }
</script>

{#if userData.loading}
  <PageLoading message="Loading user..." />
{:else if userData.error}
  <FormError message={userData.error} />
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
        <Pill accent={getUserRoleAccent(user.role)}>{user.role}</Pill>
        <Pill accent={getUserStatusAccent(user.status)}>{user.status}</Pill>
      </DetailMeta>
    {/snippet}

    {#snippet actions()}
      <DropdownMenu items={getUserMenuItems(user)}>
        {#snippet trigger()}
          <MoreVertical size={16} aria-hidden="true" />
        {/snippet}
      </DropdownMenu>
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
          <FormError message={sessionsData.error} />
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
                <Pill accent={getSessionStatusAccent(value)}>{value}</Pill>
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
          <FormError message={activityData.error} />
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
                <Pill accent={getActivityAccent(value)}>{value}</Pill>
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
      <Field label="Role">
        <Select
          value={selectedRole}
          onchange={(v) => { selectedRole = v as UserRole; }}
          items={roleItems}
          placeholder="Select role"
          disabled={submitting}
        />
      </Field>

      <FormActions align="end">
        {#snippet danger()}
          <TextButton type="button" onclick={() => (showRoleDialog = false)} disabled={submitting}>
            Cancel
          </TextButton>
        {/snippet}
        <Button type="submit" variant="primary" disabled={submitting}>
          {submitting ? "Saving..." : "Save"}
        </Button>
      </FormActions>
    </form>
  {/snippet}
</FormDialog>

<AlertDialog
  bind:open={showRevokeDialog}
  title="Revoke session"
  description="This will immediately log the user out of this session. They will need to log in again."
  confirmLabel={revokingSession ? "Revoking..." : "Revoke"}
  showTrigger={false}
  onConfirm={handleRevokeSession}
  onCancel={() => (sessionToRevoke = null)}
/>

