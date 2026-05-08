<script lang="ts">
import {
  type SpaFormResult,
  SpaFormShell
} from "@decodelabs/underlay/patterns";
import type { NightfireValue } from "@decodelabs/underlay/nightfire/validation";
import {
  consumeNavigationContext
} from "@decodelabs/underlay/runtime/navigation";
import {
  goto } from "$app/navigation";
  import { adminCommands,
  type Category } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { extractApiError } from "$lib/utils/api-errors";
  import ProjectForm from "$lib/forms/ProjectForm.svelte";
  
  // Navigation context
  const defaultBackHref = "/projects";
  const { backInfo, returnTo } = consumeNavigationContext("Back to projects", defaultBackHref);

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
    const formIntent = String(formData.get("intent") ?? "save-close");
    const formReturnTo = String(formData.get("returnTo") ?? "").trim() || null;

    const buildValues = () => ({
      name,
      description,
      categoryId,
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
      const project = await adminCommands.createProject(
        { name, description, categoryId },
        fetch,
        token
      );

      if (formIntent === "save-close") {
        const redirectTarget = formReturnTo && formReturnTo.startsWith("/")
          ? formReturnTo
          : "/projects";
        return { success: true, redirectTo: redirectTarget };
      }

      // After save, redirect to edit page for the new project
      return { success: true, redirectTo: `/projects/${project.id}/edit` };
    } catch (e) {
      const { message, fieldErrors: apiFieldErrors } = extractApiError(e, "Failed to create project");
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

<SpaFormShell
  section="New Project"
  subtitle="Create a new project to organize your tasks"
  backHref={backInfo.href}
  backLabel={backInfo.label}
  backIsContextual={backInfo.isContextual ?? false}
  success={success === true}
  successMessage="Project created successfully."
  error={success === false && !fieldErrors ? error : null}
  {fieldErrors}
  onSubmit={handleSubmit}
  onResult={handleResult}
  navigate={goto}
>
  <ProjectForm
    mode="create"
    {fetchCategories}
    createCategory={createCategoryInline}
    values={{
      name: typeof formValues?.name === "string" ? formValues.name : "",
      description: isNightfireValue(formValues?.description) ? formValues.description : null,
      categoryId: typeof formValues?.categoryId === "string" ? formValues.categoryId : null,
      status: "active"
    }}
    errors={fieldErrors}
    cancelHref={backInfo.href}
    {returnTo}
    bind:intent
  />
</SpaFormShell>
