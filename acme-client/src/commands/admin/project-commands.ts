import type { ListResponse, SingleResponse } from "../../types/common-types.js";
import type {
  Project,
  ProjectWithCounts,
  CreateProjectPayload,
  UpdateProjectPayload,
  ReorderPayload,
  ReorderResult,
  BatchDeletePayload,
  BatchDeleteResult,
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
 * List projects with counts (admin).
 *
 * Supports filtering and sorting via QueryParams:
 * - `sort=name:asc,weight:desc,categoryName:asc`
 * - `filter[categoryId]=<uuid>`
 * - `filter[status]=active`
 * - `filter[ownerId]=<uuid>`
 */
export async function listProjects(
  fetchFn: typeof fetch,
  accessToken: string,
  query?: QueryParams
): Promise<ProjectWithCounts[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const path = appendQueryParams("/v1/admin/projects", toSnakeQueryParams(query));
  const response = await http.get<ListResponse<ProjectWithCounts>>(path);
  return response.data;
}

/**
 * List projects for suggestions (RelationSelector).
 */
export async function listProjectsForSuggestions(
  fetchFn: typeof fetch,
  accessToken: string,
  options?: SuggestionRequestOptions
): Promise<Project[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const path = appendSuggestionParams("/v1/admin/projects", options);
  const response = await http.get<ListResponse<Project>>(path);
  return response.data;
}

/**
 * Get a single project by ID.
 */
export async function getProject(
  projectId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Project> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<Project>>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}`
  );
  return response.data;
}

/**
 * Create a new project.
 */
export async function createProject(
  payload: CreateProjectPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Project> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<Project>>(
    "/v1/admin/projects",
    payload
  );
  return response.data;
}

/**
 * Update an existing project.
 */
export async function updateProject(
  projectId: string,
  payload: UpdateProjectPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Project> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.patch<SingleResponse<Project>>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}`,
    payload
  );
  return response.data;
}

/**
 * Soft delete a project.
 */
export async function softDeleteProject(
  projectId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.delete(`/v1/admin/projects/${encodeURIComponent(projectId)}`);
}

/**
 * Restore a soft-deleted project.
 */
export async function restoreProject(
  projectId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Project> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<Project>>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/restore`,
    {}
  );
  return response.data;
}

/**
 * Reorder projects.
 */
export async function reorderProjects(
  payload: ReorderPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<ReorderResult> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  return await http.put<ReorderResult>("/v1/admin/projects/reorder", payload);
}

/**
 * Batch delete projects.
 */
export async function batchDeleteProjects(
  payload: BatchDeletePayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<BatchDeleteResult> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  return await http.post<BatchDeleteResult>(
    "/v1/admin/projects:batch-delete",
    payload
  );
}
