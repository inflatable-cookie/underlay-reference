import type { ListResponse, SingleResponse } from "../../types/common-types.js";
import type {
  Category,
  CategoryWithCounts,
  CreateCategoryPayload,
  UpdateCategoryPayload,
  ReorderPayload,
  ReorderResult,
} from "../../types/admin-types.js";
import { getAdminHttpClient } from "../../utils/client-factory.js";
import {
  appendQueryParams,
  type QueryParams,
} from "@decodelabs/underlay/client";
import {
  appendSuggestionParams,
  type SuggestionRequestOptions,
} from "@decodelabs/underlay/patterns";
import { toSnakeQueryParams } from "./utils.js";

/**
 * List categories with counts (admin).
 *
 * Supports filtering and sorting via QueryParams:
 * - `sort=name:asc,weight:desc`
 * - `filter[isActive]=true`
 */
export async function listCategories(
  fetchFn: typeof fetch,
  accessToken: string,
  query?: QueryParams
): Promise<CategoryWithCounts[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const path = appendQueryParams("/v1/admin/categories", toSnakeQueryParams(query));
  const response = await http.get<ListResponse<CategoryWithCounts>>(path);
  return response.data;
}

/**
 * List categories for suggestions (RelationSelector).
 */
export async function listCategoriesForSuggestions(
  fetchFn: typeof fetch,
  accessToken: string,
  options?: SuggestionRequestOptions
): Promise<Category[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const path = appendSuggestionParams("/v1/admin/categories", options);
  const response = await http.get<ListResponse<Category>>(path);
  return response.data;
}

/**
 * Get a single category by ID.
 */
export async function getCategory(
  categoryId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Category> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<Category>>(
    `/v1/admin/categories/${encodeURIComponent(categoryId)}`
  );
  return response.data;
}

/**
 * Create a new category.
 */
export async function createCategory(
  payload: CreateCategoryPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Category> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<Category>>(
    "/v1/admin/categories",
    payload
  );
  return response.data;
}

/**
 * Update an existing category.
 */
export async function updateCategory(
  categoryId: string,
  payload: UpdateCategoryPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Category> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.patch<SingleResponse<Category>>(
    `/v1/admin/categories/${encodeURIComponent(categoryId)}`,
    payload
  );
  return response.data;
}

/**
 * Soft delete a category.
 */
export async function softDeleteCategory(
  categoryId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.delete(`/v1/admin/categories/${encodeURIComponent(categoryId)}`);
}

/**
 * Reorder categories.
 */
export async function reorderCategories(
  payload: ReorderPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<ReorderResult> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  return await http.put<ReorderResult>("/v1/admin/categories/reorder", payload);
}
