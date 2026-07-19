<script lang="ts">
import {
  type SpaFormResult
} from "@decodelabs/underlay/patterns";
import { EntityFormPage } from "@decodelabs/underlay/templates";
import {
  consumeNavigationContext
} from "@decodelabs/underlay/runtime/navigation";
import { slugify } from "@decodelabs/underlay/utils/slug";
import {
  goto } from "$app/navigation";
  import { adminCommands,
  type Category } from "@api-client";
  import { auth } from "$lib/stores/auth";
  import { resolveRedirectTo } from "@decodelabs/underlay/client/route-protection";
  import { extractApiError } from "$lib/utils/api-errors";
  import CategoryForm from "$lib/forms/CategoryForm.svelte";
    
  // Navigation context
  const defaultBackHref = "/categories";
  const { backInfo, returnTo } = consumeNavigationContext("Back to categories", defaultBackHref);

  // Form state
  let success = $state<boolean | null>(null);
  let error = $state<string | null>(null);
  let fieldErrors = $state<Record<string, string> | null>(null);
  let formValues = $state<Record<string, unknown> | undefined>(undefined);
  let intent = $state<"save" | "save-close">("save-close");

  // Slug validation function
  async function validateSlug(slug: string) {
    const token = auth.getToken();
    if (!token) return { valid: false, message: "Not authenticated" };

    return await adminCommands.validateField(
      {
        entity: "category",
        field: "slug",
        value: slug
      },
      window.fetch.bind(window),
      token
    );
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
    const slug = String(formData.get("slug") ?? "").trim() || slugify(name);
    const description = String(formData.get("description") ?? "").trim() || null;
    const color = String(formData.get("color") ?? "#6366f1").trim();
    const formIntent = String(formData.get("intent") ?? "save-close");
    const formReturnTo = String(formData.get("returnTo") ?? "").trim() || null;

    const buildValues = () => ({
      name,
      slug,
      description,
      color,
      intent: formIntent
    });

    // Validate required fields
    const errors: Record<string, string> = {};
    if (!name) errors.name = "Name is required";
    if (!slug) errors.slug = "Slug is required";

    if (Object.keys(errors).length > 0) {
      return {
        success: false,
        error: "Please fill in all required fields",
        fieldErrors: errors,
        values: buildValues()
      };
    }

    try {
      const category = await adminCommands.createCategory(
        { name, slug, description, color },
        fetch,
        token
      );

      if (formIntent === "save-close") {
        const redirectTarget = resolveRedirectTo(formReturnTo, "/categories");
        return { success: true, redirectTo: redirectTarget };
      }

      // After save, redirect to edit page for the new category
      return { success: true, redirectTo: `/categories/${category.id}/edit` };
    } catch (e) {
      const { message, fieldErrors: apiFieldErrors } = extractApiError(e, "Failed to create category");
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

<EntityFormPage
  section="Categories"
  title="New category"
  subtitle="Create a new category to organize your projects"
  backHref={backInfo.href}
  backLabel={backInfo.label}
  backIsContextual={backInfo.isContextual ?? false}
  success={success === true}
  successMessage="Category created successfully."
  error={success === false && !fieldErrors ? error : null}
  {fieldErrors}
  onSubmit={handleSubmit}
  onResult={handleResult}
  navigate={goto}
>
  <CategoryForm
    mode="create"
    validateSlug={validateSlug}
    values={{
      name: typeof formValues?.name === "string" ? formValues.name : "",
      slug: typeof formValues?.slug === "string" ? formValues.slug : "",
      description: typeof formValues?.description === "string" ? formValues.description : "",
      color: typeof formValues?.color === "string" ? formValues.color : "#6366f1",
      isActive: true
    }}
    errors={fieldErrors}
    cancelHref={backInfo.href}
    {returnTo}
    bind:intent
  />
</EntityFormPage>
