<script lang="ts">
  import { untrack } from "svelte";
  import type { PageData } from "./$types";
  import type { Category } from "acme-client";
  import { adminCommands } from "acme-client";
  import { auth } from "$lib/stores/auth";
  import { extractApiError } from "$lib/utils/api-errors";
  import CategoryForm from "$lib/forms/CategoryForm.svelte";
  import { goto } from "$app/navigation";
  import {
    SpaFormShell,
    PageHeaderMeta,
    PageHeaderMetaRow,
    PageHeaderMetaItem,
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

  // Fetch category data
  const pageData = useAuthenticatedData(
    async (fetch, token) => {
      const category = await adminCommands.getCategory(data.categoryId, fetch, token);
      if (!category) {
        throw new Error("Category not found");
      }
      return { category };
    },
    {
      defaultValue: { category: null as Category | null }
    }
  );

  const category = $derived(pageData.data?.category);

  // Navigation context
  const defaultBackHref = untrack(() => `/categories/${data.categoryId}`);
  const { backInfo, returnTo } = consumeNavigationContext("Back to category", defaultBackHref);

  // Compute dynamic back info based on loaded data
  const computedBackInfo = $derived(
    computeBackInfo(backInfo, category ? {
      href: `/categories/${category.id}`,
      label: "Back to category"
    } : undefined)
  );

  // Form state
  let success = $state<boolean | null>(null);
  let error = $state<string | null>(null);
  let fieldErrors = $state<Record<string, string> | null>(null);
  let formValues = $state<Record<string, unknown> | undefined>(undefined);
  let intent = $state<"save" | "save-close">("save-close");

  // Slug validation function
  async function validateSlug(slug: string) {
    if (!category) return { valid: false, message: "Data not loaded" };
    const token = auth.getToken();
    if (!token) return { valid: false, message: "Not authenticated" };

    return await adminCommands.validateField(
      {
        entity: "category",
        field: "slug",
        value: slug,
        exclude_id: category.id
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
    const slug = String(formData.get("slug") ?? "").trim();
    const description = String(formData.get("description") ?? "").trim() || null;
    const color = String(formData.get("color") ?? "#6366f1").trim();
    const isActive = formData.get("isActive") === "on";
    const formIntent = String(formData.get("intent") ?? "save-close");
    const formReturnTo = String(formData.get("returnTo") ?? "").trim() || null;

    const buildValues = () => ({
      name,
      slug,
      description,
      color,
      isActive,
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
      await adminCommands.updateCategory(
        data.categoryId,
        { name, slug, description, color, isActive },
        fetch,
        token
      );

      if (formIntent === "save-close") {
        const redirectTarget = formReturnTo && formReturnTo.startsWith("/")
          ? formReturnTo
          : `/categories/${data.categoryId}`;
        return { success: true, redirectTo: redirectTarget };
      }

      return { success: true, values: buildValues() };
    } catch (e) {
      const { message, fieldErrors: apiFieldErrors } = extractApiError(e, "Failed to update category");
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
  <PageLoading message="Loading category..." />
{:else if pageData.error}
  <FormError message={pageData.error} />
{:else if category}
  {#snippet headerMeta()}
    <PageHeaderMeta>
      <PageHeaderMetaRow>
        <PageHeaderMetaItem label="ID">
          <Code copy>{category.id}</Code>
        </PageHeaderMetaItem>
      </PageHeaderMetaRow>
    </PageHeaderMeta>
  {/snippet}

  <SpaFormShell
    section="Edit Category"
    subtitle={category.name}
    backHref={computedBackInfo.href}
    backLabel={computedBackInfo.label}
    backIsContextual={computedBackInfo.isContextual ?? false}
    bannerMessage={!category.isActive ? "This category is inactive." : undefined}
    success={success === true}
    successMessage="Category updated successfully."
    error={success === false && !fieldErrors ? error : null}
    {fieldErrors}
    {headerMeta}
    onSubmit={handleSubmit}
    onResult={handleResult}
    navigate={goto}
  >
    <CategoryForm
      mode="edit"
      categoryId={category.id}
      {validateSlug}
      values={{
        name: typeof formValues?.name === "string" ? formValues.name : category.name,
        slug: typeof formValues?.slug === "string" ? formValues.slug : category.slug,
        description: typeof formValues?.description === "string" ? formValues.description : category.description ?? "",
        color: typeof formValues?.color === "string" ? formValues.color : category.color ?? "#6366f1",
        isActive: typeof formValues?.isActive === "boolean" ? formValues.isActive : category.isActive
      }}
      errors={fieldErrors}
      cancelHref={computedBackInfo.href}
      {returnTo}
      bind:intent
    />
  </SpaFormShell>
{/if}
