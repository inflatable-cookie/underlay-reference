<script lang="ts">
  import { Callout as PoodleCallout } from "@poodle/svelte-primitives";
  import { untrack } from "svelte";
  import { goto } from "$app/navigation";
  import type { PageData } from "./$types";
  import { adminCommands, type UserDetail, type UserRole, type UserStatus } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { extractApiError, isPreconditionFailed } from "$lib/utils/api-errors";
  import UserForm from "$lib/forms/UserForm.svelte";
  import {
    SpaFormShell,
    DetailMeta,
    DetailMetaId,
    DetailMetaItem,
    DetailMetaSeparator,
    computeBackInfo,
    consumeNavigationContext,
    useAuthenticatedData,
    type SpaFormResult
  } from "@decodelabs/underlay/patterns";
  import { Code, PageLoading } from "@decodelabs/underlay/components";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const { data: user, etag } = await adminCommands.getUserWithEtag(data.userId, fetch, token);
      return { user, etag };
    },
    {
      defaultValue: { user: null as UserDetail | null, etag: null as string | null }
    }
  );

  const user = $derived(pageData.data?.user);
  let currentEtag = $state<string | null>(null);

  $effect(() => {
    if (pageData.data?.etag) {
      currentEtag = pageData.data.etag;
    }
  });

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
      const result = await adminCommands.updateUserWithEtag(
        user.id,
        { email, displayName, role: role as UserRole, status: status as UserStatus },
        fetch,
        token,
        { ifMatch: currentEtag ?? undefined }
      );
      currentEtag = result.etag;

      if (formIntent === "save-close") {
        const redirectTarget = formReturnTo && formReturnTo.startsWith("/")
          ? formReturnTo
          : `/users/${user.id}`;
        return { success: true, redirectTo: redirectTarget };
      }

      return { success: true, values: buildValues() };
    } catch (e) {
      if (isPreconditionFailed(e)) {
        const latest = await adminCommands.getUserWithEtag(user.id, fetch, token);
        currentEtag = latest.etag;
        formValues = {
          email: latest.data.email,
          displayName: latest.data.displayName ?? "",
          role: latest.data.role,
          status: latest.data.status,
          intent: formIntent
        };
        await pageData.refetch();
        return {
          success: false,
          error: "This user was changed in another session. Review the latest values, reapply your edits, and save again.",
          values: formValues
        };
      }
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
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if user}
  {#snippet headerMeta()}
    <DetailMeta>
      <DetailMetaId value={user.id} />
      <DetailMetaSeparator />
      <DetailMetaItem label="Email">
        <Code copy>{user.email}</Code>
      </DetailMetaItem>
    </DetailMeta>
  {/snippet}

  <SpaFormShell
    section="Edit User"
    subtitle={user.email}
    backHref={computedBackInfo.href}
    backLabel={computedBackInfo.label}
    backIsContextual={computedBackInfo.isContextual ?? false}
    bannerMessage={user.status !== "active" ? `User status: ${user.status}` : undefined}
    {success}
    successMessage="User updated successfully."
    {error}
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
