<script lang="ts">
  import { goto } from "$app/navigation";
  import { untrack } from "svelte";
  import type { PageData } from "./$types";
  import { EntityFormPage } from "@inflatable-cookie/underlay/templates";
  import type { SpaFormResult } from "@inflatable-cookie/underlay/patterns";
  import { useAuthenticatedData } from "@inflatable-cookie/underlay/runtime/auth";
  import { consumeNavigationContext } from "@inflatable-cookie/underlay/runtime/navigation";
  import { adminCommands, type Project } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { extractApiError } from "$lib/utils/api-errors";
  import LabelForm from "$lib/forms/LabelForm.svelte";

  let { data }: { data: PageData } = $props();

  let nameValue = $state("");
  let colorValue = $state("#6366f1");
  let submitting = $state(false);
  let fieldErrors = $state<Record<string, string> | null>(null);

  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const project = await adminCommands.getProject(data.projectId, fetch, token);
      return { project };
    },
    {
      defaultValue: { project: null as Project | null }
    }
  );

  const project = $derived(pageData.data?.project);

  const defaultBackHref = untrack(() => `/projects/${data.projectId}/labels`);
  const { backInfo } = consumeNavigationContext("Back to labels", defaultBackHref);

  async function handleSubmit(formData: FormData): Promise<SpaFormResult> {
    const token = auth.getToken();
    if (!token) {
      return { success: false, error: "Not authenticated" };
    }

    const name = String(formData.get("name") ?? "").trim();
    const color = String(formData.get("color") ?? "").trim();

    const errors: Record<string, string> = {};
    if (!name) errors.name = "Name is required";

    if (Object.keys(errors).length > 0) {
      return {
        success: false,
        error: "Please fill in all required fields",
        fieldErrors: errors
      };
    }

    submitting = true;

    try {
      const label = await adminCommands.createLabel(
        data.projectId,
        { name, color: color || undefined },
        fetch,
        token
      );

      return {
        success: true,
        redirectTo: `/projects/${data.projectId}/labels/${label.id}`
      };
    } catch (e) {
      const { message, fieldErrors: apiFieldErrors } = extractApiError(e, "Failed to create label");
      return {
        success: false,
        error: message,
        fieldErrors: apiFieldErrors
      };
    } finally {
      submitting = false;
    }
  }

  function handleResult(result: SpaFormResult) {
    fieldErrors = result.fieldErrors ?? null;
  }
</script>

<EntityFormPage
  section="Labels"
  title="New label"
  subtitle={project ? `For project: ${project.name}` : undefined}
  backHref={backInfo.href}
  backLabel={backInfo.label}
  backIsContextual={backInfo.isContextual ?? false}
  loading={pageData.loading}
  loadingMessage="Loading project..."
  error={pageData.error}
  {fieldErrors}
  onSubmit={handleSubmit}
  onResult={handleResult}
  navigate={goto}
>
  <LabelForm
    mode="create"
    bind:name={nameValue}
    bind:color={colorValue}
    errors={fieldErrors}
    {submitting}
    cancelHref={backInfo.href}
    onCancel={() => goto(backInfo.href)}
  />
</EntityFormPage>
