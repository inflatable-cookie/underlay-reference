<script lang="ts">
  import { untrack } from "svelte";
  import { goto } from "$app/navigation";
  import type { PageData } from "./$types";
  import { adminCommands, type UserDetail, type UserRole, type UserStatus } from "@api-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import { extractApiError } from "$lib/utils/api-errors";
  import UserForm from "$lib/forms/UserForm.svelte";
  import {
    SpaFormShell,
    PageHeaderMeta,
    PageHeaderMetaRow,
    PageHeaderMetaItem,
    PageHeaderMetaSeparator,
    computeBackInfo,
    consumeNavigationContext,
    useAuthenticatedData,
    type SpaFormResult
  } from "@decodelabs/underlay/patterns";
  import { Code, FormError, PageLoading } from "@decodelabs/underlay/components";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const user = await adminCommands.getUser(data.userId, fetch, token);
      return { user };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { user: null as UserDetail | null }
    }
  );

  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  const user = $derived(pageData.data?.user);

  const defaultBackHref = untrack(() => `/users/${data.userId}`);
  const { backInfo, returnTo } = consumeNavigationContext("Back to user", defaultBackHref);

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

  let success = $state<boolean | null>(null);
  let error = $state<string | null>(null);
  let fieldErrors = $state<Record<string, string> | null>(null);
  let formValues = $state<Record<string, unknown> | undefined>(undefined);
  let intent = $state<"save" | "save-close">("save-close");

  async function handleSubmit(formData: FormData): Promise<SpaFormResult> {
    const token = auth.getToken();
    if (!token) {
      return { success: false, error: "Not authenticated" };
    }

    if (!user) {
      return { success: false, error: "Data not loaded" };
    }

    const email = String(formData.get("email") ?? "").trim();
    const displayName = String(formData.get("displayName") ?? "").trim() || null;
    const role = String(formData.get("role") ?? user.role).trim();
    const status = String(formData.get("status") ?? user.status).trim();
    const formIntent = String(formData.get("intent") ?? "save-close");
    const formReturnTo = String(formData.get("returnTo") ?? "").trim() || null;

    const buildValues = () => ({
      email,
      displayName,
      role,
      status,
      intent: formIntent
    });

    const errors: Record<string, string> = {};
    if (!email) errors.email = "Email is required";
    if (email && !email.includes("@")) errors.email = "Email must be valid";
    if (!role) errors.role = "Role is required";
    if (!status) errors.status = "Status is required";

    if (Object.keys(errors).length > 0) {
      return {
        success: false,
        error: "Please fill in all required fields",
        fieldErrors: errors,
        values: buildValues()
      };
    }

    try {
      await adminCommands.updateUser(
        user.id,
        { email, displayName, role: role as UserRole, status: status as UserStatus },
        fetch,
        token
      );

      if (formIntent === "save-close") {
        const redirectTarget = formReturnTo && formReturnTo.startsWith("/")
          ? formReturnTo
          : `/users/${user.id}`;
        return { success: true, redirectTo: redirectTarget };
      }

      return { success: true, values: buildValues() };
    } catch (e) {
      const { message, fieldErrors: apiFieldErrors } = extractApiError(e, "Failed to update user");
      return {
        success: false,
        error: message,
        fieldErrors: apiFieldErrors,
        values: buildValues()
      };
    }
  }

  function handleResult(result: SpaFormResult) {
    success = result.success;
    error = result.error ?? null;
    fieldErrors = result.fieldErrors ?? null;
    formValues = result.values as Record<string, unknown> | undefined;
  }
</script>

{#if pageData.loading}
  <PageLoading message="Loading user..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if user}
  {#snippet headerMeta()}
    <PageHeaderMeta>
      <PageHeaderMetaRow>
        <PageHeaderMetaItem label="ID">
          <Code copy>{user.id}</Code>
        </PageHeaderMetaItem>
        <PageHeaderMetaSeparator />
        <PageHeaderMetaItem label="Email">
          <Code copy>{user.email}</Code>
        </PageHeaderMetaItem>
      </PageHeaderMetaRow>
    </PageHeaderMeta>
  {/snippet}

  <SpaFormShell
    section="Edit User"
    subtitle={user.email}
    backHref={computedBackInfo.href}
    backLabel={computedBackInfo.label}
    backIsContextual={computedBackInfo.isContextual ?? false}
    bannerMessage={user.status !== "active" ? `User status: ${user.status}` : undefined}
    success={success === true}
    successMessage="User updated successfully."
    error={success === false && !fieldErrors ? error : null}
    {fieldErrors}
    {headerMeta}
    onSubmit={handleSubmit}
    onResult={handleResult}
    navigate={goto}
  >
    <UserForm
      mode="edit"
      values={{
        email: typeof formValues?.email === "string" ? formValues.email : user.email,
        displayName: typeof formValues?.displayName === "string" ? formValues.displayName : user.displayName ?? "",
        role: typeof formValues?.role === "string" ? formValues.role : user.role,
        status: typeof formValues?.status === "string" ? formValues.status : user.status
      }}
      errors={fieldErrors}
      cancelHref={computedBackInfo.href}
      {returnTo}
      bind:intent
    />
  </SpaFormShell>
{/if}
