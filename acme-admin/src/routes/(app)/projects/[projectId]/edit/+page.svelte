<script lang="ts">
  import { untrack } from "svelte";
  import type { PageData } from "./$types";
  import type { Project, Category, CategoryWithCounts } from "acme-client";
  import { adminCommands } from "acme-client";
  import { auth, authLoading, currentUser } from "$lib/stores/auth";
  import { extractApiError } from "$lib/utils/api-errors";
  import ProjectForm from "$lib/forms/ProjectForm.svelte";
  import { goto } from "$app/navigation";
  import {
    SpaFormShell,
    computeBackInfo,
    consumeNavigationContext,
    useAuthenticatedData,
    type SpaFormResult
  } from "@decodelabs/underlay/patterns";
  import { FormError, PageLoading } from "@decodelabs/underlay/components";

  interface Props {
    data: PageData;
  }

  let { data }: Props = $props();

  // Fetch project and categories data
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const [project, categories] = await Promise.all([
        adminCommands.getProject(data.projectId, fetch, token),
        adminCommands.listCategories(fetch, token)
      ]);
      if (!project) {
        throw new Error("Project not found");
      }
      return { project, categories };
    },
    {
      getToken: () => auth.getToken(),
      defaultValue: { project: null as Project | null, categories: [] as CategoryWithCounts[] }
    }
  );

  // Trigger fetch when auth is ready
  $effect(() => {
    pageData.tryFetch($authLoading, $currentUser);
  });

  const project = $derived(pageData.data?.project);
  const categories = $derived(pageData.data?.categories ?? []);

  // Function to create categories inline
  async function createCategoryInline(
    name: string,
    slug: string,
    description: string | null,
    color: string | null
  ): Promise<Category> {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");

    const category = await adminCommands.createCategory(
      { name, slug, description, color },
      fetch,
      token
    );

    // Refresh the categories list
    await pageData.refetch();

    return category;
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

  /**
   * Handle form submission.
   */
  async function handleSubmit(formData: FormData): Promise<SpaFormResult> {
    const token = auth.getToken();
    if (!token) {
      return { success: false, error: "Not authenticated" };
    }

    const name = String(formData.get("name") ?? "").trim();
    const description = String(formData.get("description") ?? "").trim() || null;
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
      await adminCommands.updateProject(
        data.projectId,
        { name, description, categoryId, status },
        fetch,
        token
      );

      if (formIntent === "save-close") {
        const redirectTarget = formReturnTo && formReturnTo.startsWith("/")
          ? formReturnTo
          : `/projects/${data.projectId}`;
        return { success: true, redirectTo: redirectTarget };
      }

      return { success: true, values: buildValues() };
    } catch (e) {
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
  <PageLoading message="Loading project..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if project}
  {#snippet headerMeta()}
    <p><strong>ID:</strong> <code>{project.id}</code></p>
  {/snippet}

  <SpaFormShell
    title="Edit Project"
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
      {categories}
      createCategory={createCategoryInline}
      values={{
        name: typeof formValues?.name === "string" ? formValues.name : project.name,
        description: typeof formValues?.description === "string" ? formValues.description : project.description ?? "",
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
