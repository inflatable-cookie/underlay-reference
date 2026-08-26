import type { PagedListResponse, SingleResponse } from "../../types/common-types.js";
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
} from "@inflatable-cookie/underlay/client/query";
import {
  getHeaderValueCaseInsensitive,
  toSnakeQueryParams,
  type WithEtag,
} from "./utils.js";

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
): Promise<PagedListResponse<ProjectWithCounts>> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const path = appendQueryParams("/v1/admin/projects", toSnakeQueryParams(query));
  return await http.get<PagedListResponse<ProjectWithCounts>>(path);
}

/**
 * Get a single project by ID.
 */
export async function getProject(
  projectId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Project> {
  const result = await getProjectWithEtag(projectId, fetchFn, accessToken);
  return result.data;
}

export async function getProjectWithEtag(
  projectId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<WithEtag<Project>> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.getWithMeta<SingleResponse<Project>>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}`
  );
  return {
    data: response.body!.data,
    etag: getHeaderValueCaseInsensitive(response.headers, "etag"),
  };
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
  const result = await updateProjectWithEtag(projectId, payload, fetchFn, accessToken);
  return result.data;
}

export async function updateProjectWithEtag(
  projectId: string,
  payload: UpdateProjectPayload,
  fetchFn: typeof fetch,
  accessToken: string,
  options?: { ifMatch?: string }
): Promise<WithEtag<Project>> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const headers = options?.ifMatch ? { "If-Match": options.ifMatch } : undefined;
  const response = await http.patchWithMeta<SingleResponse<Project>>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}`,
    payload,
    headers
  );
  return {
    data: response.body!.data,
    etag: getHeaderValueCaseInsensitive(response.headers, "etag"),
  };
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
