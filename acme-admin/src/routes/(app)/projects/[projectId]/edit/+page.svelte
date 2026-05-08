<script lang="ts">
import { type SpaFormResult, SpaFormShell } from "@decodelabs/underlay/patterns";
import type { NightfireValue } from "@decodelabs/underlay/nightfire/validation";
import {
  useAuthenticatedData
} from "@decodelabs/underlay/runtime/auth";
import {
  computeBackInfo,
  consumeNavigationContext
} from "@decodelabs/underlay/runtime/navigation";
import {
  Callout as PoodleCallout,
  Code as PoodleCode,
  MetaBar as PoodleMetaBar,
  MetaItem as PoodleMetaItem } from "@poodle/svelte";
  import { untrack } from "svelte";
  import type { PageData } from "./$types";
  import type { Project,
  Category } from "@api-client";
  import { adminCommands } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { extractApiError,
  isPreconditionFailed } from "$lib/utils/api-errors";
  import ProjectForm from "$lib/forms/ProjectForm.svelte";
  import { goto } from "$app/navigation";
    import { PageLoading } from "@poodle/svelte";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  // Fetch project data
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const { data: project, etag } = await adminCommands.getProjectWithEtag(data.projectId, fetch, token);
      if (!project) {
        throw new Error("Project not found");
      }
      return { project, etag };
    },
    {
      defaultValue: { project: null as Project | null, etag: null as string | null }
    }
  );

  const project = $derived(pageData.data?.project);
  let currentEtag = $state<string | null>(null);

  $effect(() => {
    if (pageData.data?.etag) {
      currentEtag = pageData.data.etag;
    }
  });

  // Lazy-load categories for the RelationSelector
  async function fetchCategories(): Promise<Category[]> {
    const token = auth.getToken();
    if (!token) return [];
    return adminCommands.listCategoriesForSuggestions(fetch, token);
  }

  // Function to create categories inline
  async function createCategoryInline(
    name: string,
    slug: string,
    description: string | null,
    color: string | null
  ): Promise<Category> {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");

    return adminCommands.createCategory(
      { name, slug, description, color },
      fetch,
      token
    );
  }

  // Navigation context
  const defaultBackHref = untrack(() => `/projects/${data.projectId}`);
  const { backInfo, returnTo } = consumeNavigationContext("Back to project", defaultBackHref);

  // Compute dynamic back info based on loaded data
  const computedBackInfo = $derived(
    computeBackInfo(backInfo, project ? {
      href: `/projects/${project.id}`,
      label: "Back to project"
    } : undefined)
  );

  // Form state
  let success = $state<boolean | null>(null);
  let error = $state<string | null>(null);
  let fieldErrors = $state<Record<string, string> | null>(null);
  let formValues = $state<Record<string, unknown> | undefined>(undefined);
  let intent = $state<"save" | "save-close">("save-close");

  function parseNightfireField(formData: FormData, name: string): NightfireValue | null {
    const raw = String(formData.get(name) ?? "").trim();
    if (!raw) return null;

    try {
      return JSON.parse(raw) as NightfireValue;
    } catch {
      throw new Error(`Invalid ${name} payload`);
    }
  }

  function isNightfireValue(value: unknown): value is NightfireValue {
    return !!value && typeof value === "object" && "schema" in value;
  }

  /**
   * Handle form submission.
   */
  async function handleSubmit(formData: FormData): Promise<SpaFormResult> {
    const token = auth.getToken();
    if (!token) {
      return { success: false, error: "Not authenticated" };
    }

    const name = String(formData.get("name") ?? "").trim();
    const description = parseNightfireField(formData, "description");
    const categoryId = String(formData.get("categoryId") ?? "").trim() || null;
    const status = String(formData.get("status") ?? "active").trim();
    const formIntent = String(formData.get("intent") ?? "save-close");
    const formReturnTo = String(formData.get("returnTo") ?? "").trim() || null;

    const buildValues = () => ({
      name,
      description,
      categoryId,
      status,
      intent: formIntent
    });

// Validate required fields
    const errors: Record<string, string> = {};
    if (!name) errors.name = "Name is required";

    if (Object.keys(errors).length > 0) {
      return {
        success: false,
        error: "Please fill in all required fields",
        fieldErrors: errors,
        values: buildValues()
      };
    }

    try {
      const result = await adminCommands.updateProjectWithEtag(
        data.projectId,
        { name, description, categoryId, status },
        fetch,
        token,
        { ifMatch: currentEtag ?? undefined }
      );
      currentEtag = result.etag;

      if (formIntent === "save-close") {
        const redirectTarget = formReturnTo && formReturnTo.startsWith("/")
          ? formReturnTo
          : `/projects/${data.projectId}`;
        return { success: true, redirectTo: redirectTarget };
      }

      return { success: true, values: buildValues() };
    } catch (e) {
      if (isPreconditionFailed(e)) {
        const latest = await adminCommands.getProjectWithEtag(data.projectId, fetch, token);
        currentEtag = latest.etag;
        formValues = {
          name: latest.data.name,
          description: latest.data.description ?? null,
          categoryId: latest.data.categoryId ?? "",
          status: latest.data.status,
          intent: formIntent
        };
        await pageData.refetch();
        return {
          success: false,
          error: "This project was changed in another session. Review the latest values, reapply your edits, and save again.",
          values: formValues
        };
      }
      const { message, fieldErrors: apiFieldErrors } = extractApiError(e, "Failed to update project");
      return {
        success: false,
        error: message,
        fieldErrors: apiFieldErrors,
        values: buildValues()
      };
    }
  }

  /**
   * Handle form submission result.
   */
  function handleResult(result: SpaFormResult) {
    success = result.success;
    error = result.error ?? null;
    fieldErrors = result.fieldErrors ?? null;
    formValues = result.values as Record<string, unknown> | undefined;
  }
</script>

{#if pageData.loading}
  <PageLoading presentation="inline" message="Loading project..." />
{:else if pageData.error}
  <PoodleCallout tone="danger" message={pageData.error} announceMode="polite" />
{:else if project}
  {#snippet headerMeta()}
    <PoodleMetaBar ariaLabel="Project metadata">
      <PoodleMetaItem label="ID">
        <PoodleCode inline source={project.id} showCopyButton />
      </PoodleMetaItem>
    </PoodleMetaBar>
  {/snippet}

  <SpaFormShell
    section="Edit Project"
    subtitle={project.name}
    backHref={computedBackInfo.href}
    backLabel={computedBackInfo.label}
    backIsContextual={computedBackInfo.isContextual ?? false}
    bannerMessage={project.status === "archived" ? "This project is archived." : undefined}
    success={success === true}
    successMessage="Project updated successfully."
    error={success === false && !fieldErrors ? error : null}
    {fieldErrors}
    {headerMeta}
    onSubmit={handleSubmit}
    onResult={handleResult}
    navigate={goto}
  >
    <ProjectForm
      mode="edit"
      projectId={project.id}
      {fetchCategories}
      createCategory={createCategoryInline}
      values={{
        name: typeof formValues?.name === "string" ? formValues.name : project.name,
        description: isNightfireValue(formValues?.description) ? formValues.description : project.description ?? null,
        categoryId: typeof formValues?.categoryId === "string" ? formValues.categoryId : project.categoryId,
        status: typeof formValues?.status === "string" ? formValues.status : project.status
      }}
      errors={fieldErrors}
      cancelHref={computedBackInfo.href}
      {returnTo}
      bind:intent
    />
  </SpaFormShell>
{/if}
