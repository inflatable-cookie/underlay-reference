<script lang="ts">
import {
  type SpaFormResult
} from "@decodelabs/underlay/patterns";
import { EntityFormPage } from "@decodelabs/underlay/templates";
import {
  consumeNavigationContext
} from "@decodelabs/underlay/runtime/navigation";
import {
  goto } from "$app/navigation";
  import { adminCommands,
  type UserRole,
  type UserStatus,
  UserRole as UserRoleConst,
  UserStatus as UserStatusConst } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { extractApiError } from "$lib/utils/api-errors";
  import UserForm from "$lib/forms/UserForm.svelte";
  
  const defaultBackHref = "/users";
  const { backInfo, returnTo } = consumeNavigationContext("Back to users", defaultBackHref);

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

    const email = String(formData.get("email") ?? "").trim();
    const displayName = String(formData.get("displayName") ?? "").trim() || null;
    const role = String(formData.get("role") ?? UserRoleConst.User).trim();
    const status = String(formData.get("status") ?? UserStatusConst.Active).trim();
    const sendPasswordReset = String(formData.get("sendPasswordReset") ?? "true") === "true";
    const formIntent = String(formData.get("intent") ?? "save-close");
    const formReturnTo = String(formData.get("returnTo") ?? "").trim() || null;

    const buildValues = () => ({
      email,
      displayName,
      role,
      status,
      sendPasswordReset,
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
      const user = await adminCommands.createUser(
        { email, displayName, role: role as UserRole, status: status as UserStatus, sendPasswordReset },
        fetch,
        token
      );

      if (formIntent === "save-close") {
        const redirectTarget = formReturnTo && formReturnTo.startsWith("/")
          ? formReturnTo
          : "/users";
        return { success: true, redirectTo: redirectTarget };
      }

      return { success: true, redirectTo: `/users/${user.id}/edit` };
    } catch (e) {
      const { message, fieldErrors: apiFieldErrors } = extractApiError(e, "Failed to create user");
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

<EntityFormPage
  section="New User"
  subtitle="Create a new user account"
  backHref={backInfo.href}
  backLabel={backInfo.label}
  backIsContextual={backInfo.isContextual ?? false}
  success={success === true}
  successMessage="User created successfully."
  error={success === false && !fieldErrors ? error : null}
  {fieldErrors}
  onSubmit={handleSubmit}
  onResult={handleResult}
  navigate={goto}
>
  <UserForm
    mode="create"
    values={{
      email: typeof formValues?.email === "string" ? formValues.email : "",
      displayName: typeof formValues?.displayName === "string" ? formValues.displayName : "",
      role: typeof formValues?.role === "string" ? formValues.role : UserRoleConst.User,
      status: typeof formValues?.status === "string" ? formValues.status : UserStatusConst.Active,
      sendPasswordReset: typeof formValues?.sendPasswordReset === "boolean" ? formValues.sendPasswordReset : true
    }}
    errors={fieldErrors}
    cancelHref={backInfo.href}
    {returnTo}
    bind:intent
  />
</EntityFormPage>
