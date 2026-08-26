<script lang="ts">
  import { goto } from "$app/navigation";
  import { untrack } from "svelte";
  import type { PageData } from "./$types";
  import { EntityFormPage } from "@inflatable-cookie/underlay/templates";
  import type { SpaFormResult } from "@inflatable-cookie/underlay/patterns";
  import { useAuthenticatedData } from "@inflatable-cookie/underlay/runtime/auth";
  import { computeBackInfo, consumeNavigationContext } from "@inflatable-cookie/underlay/runtime/navigation";
  import { adminCommands, type Label, type Project } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { extractApiError, isPreconditionFailed } from "$lib/utils/api-errors";
  import LabelForm from "$lib/forms/LabelForm.svelte";
  import LabelActionsMenu from "$lib/menus/LabelActionsMenu.svelte";

  let { data }: { data: PageData } = $props();

  let nameValue = $state("");
  let colorValue = $state("#6366f1");
  let submitting = $state(false);
  let fieldErrors = $state<Record<string, string> | null>(null);
  let initialized = $state(false);
  let currentEtag = $state<string | null>(null);

  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const [labelResult, project] = await Promise.all([
        adminCommands.getLabelWithEtag(data.projectId, data.labelId, fetch, token),
        adminCommands.getProject(data.projectId, fetch, token)
      ]);
      return { label: labelResult.data, labelEtag: labelResult.etag, project };
    },
    {
      defaultValue: {
        label: null as Label | null,
        labelEtag: null as string | null,
        project: null as Project | null
      }
    }
  );

  const label = $derived(pageData.data?.label);
  const project = $derived(pageData.data?.project);

  $effect(() => {
    if (pageData.data?.labelEtag) {
      currentEtag = pageData.data.labelEtag;
    }
  });

  $effect(() => {
    if (!label || initialized) return;
    nameValue = label.name;
    colorValue = label.color || "#6366f1";
    initialized = true;
  });

  const defaultBackHref = untrack(() => `/projects/${data.projectId}/labels/${data.labelId}`);
  const { backInfo } = consumeNavigationContext("Back to label", defaultBackHref);
  const computedBackInfo = $derived(
    computeBackInfo(
      backInfo,
      label
        ? {
            href: `/projects/${data.projectId}/labels/${label.id}`,
            label: `Back to ${label.name}`
          }
        : undefined
    )
  );

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
      const result = await adminCommands.updateLabelWithEtag(
        data.projectId,
        data.labelId,
        { name, color: color || undefined },
        fetch,
        token,
        { ifMatch: currentEtag ?? undefined }
      );

      currentEtag = result.etag;

      return {
        success: true,
        redirectTo: `/projects/${data.projectId}/labels/${data.labelId}`
      };
    } catch (e) {
      if (isPreconditionFailed(e)) {
        const latest = await adminCommands.getLabelWithEtag(data.projectId, data.labelId, fetch, token);
        currentEtag = latest.etag;
        nameValue = latest.data.name;
        colorValue = latest.data.color || "#6366f1";

        return {
          success: false,
          error: "This label was changed in another session. Review the latest values, reapply your edits, and save again."
        };
      }

      const { message, fieldErrors: apiFieldErrors } = extractApiError(e, "Failed to update label");
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

{#snippet headerActions()}
  {#if label}
    <LabelActionsMenu label={label} />
  {/if}
{/snippet}

<EntityFormPage
  section="Labels"
  title="Edit label"
  subtitle={project ? `For project: ${project.name}` : undefined}
  backHref={computedBackInfo.href}
  backLabel={computedBackInfo.label}
  backIsContextual={computedBackInfo.isContextual ?? false}
  loading={pageData.loading}
  loadingMessage="Loading label..."
  error={pageData.error}
  {fieldErrors}
  {headerActions}
  onSubmit={handleSubmit}
  onResult={handleResult}
  navigate={goto}
>
  {#if label}
    <LabelForm
      mode="edit"
      bind:name={nameValue}
      bind:color={colorValue}
      errors={fieldErrors}
      {submitting}
      cancelHref={computedBackInfo.href}
      onCancel={() => goto(computedBackInfo.href)}
    />
  {/if}
</EntityFormPage>
