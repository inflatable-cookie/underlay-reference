import type { ListResponse, PagedListResponse, SingleResponse } from "../../types/common-types.js";
import type {
  Label,
  CreateLabelPayload,
  UpdateLabelPayload,
  SetLabelsPayload,
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
 * List labels for a project (admin).
 *
 * Supports filtering and sorting via QueryParams:
 * - `sort=weight:asc,name:asc`
 * - `filter[name][like]=%search%`
 */
export async function listLabels(
  projectId: string,
  fetchFn: typeof fetch,
  accessToken: string,
  query?: QueryParams
): Promise<PagedListResponse<Label>> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const path = appendQueryParams(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/labels`,
    toSnakeQueryParams(query)
  );
  return await http.get<PagedListResponse<Label>>(path);
}

/**
 * Get a single label by ID.
 */
export async function getLabel(
  projectId: string,
  labelId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Label> {
  const result = await getLabelWithEtag(projectId, labelId, fetchFn, accessToken);
  return result.data;
}

export async function getLabelWithEtag(
  projectId: string,
  labelId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<WithEtag<Label>> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.getWithMeta<SingleResponse<Label>>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/labels/${encodeURIComponent(labelId)}`
  );
  return {
    data: response.body!.data,
    etag: getHeaderValueCaseInsensitive(response.headers, "etag"),
  };
}

/**
 * Create a new label for a project.
 */
export async function createLabel(
  projectId: string,
  payload: CreateLabelPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Label> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<Label>>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/labels`,
    payload
  );
  return response.data;
}

/**
 * Update an existing label.
 */
export async function updateLabel(
  projectId: string,
  labelId: string,
  payload: UpdateLabelPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Label> {
  const result = await updateLabelWithEtag(
    projectId,
    labelId,
    payload,
    fetchFn,
    accessToken
  );
  return result.data;
}

export async function updateLabelWithEtag(
  projectId: string,
  labelId: string,
  payload: UpdateLabelPayload,
  fetchFn: typeof fetch,
  accessToken: string,
  options?: { ifMatch?: string }
): Promise<WithEtag<Label>> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const headers = options?.ifMatch ? { "If-Match": options.ifMatch } : undefined;
  const response = await http.patchWithMeta<SingleResponse<Label>>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/labels/${encodeURIComponent(labelId)}`,
    payload,
    headers
  );
  return {
    data: response.body!.data,
    etag: getHeaderValueCaseInsensitive(response.headers, "etag"),
  };
}

/**
 * Soft delete a label.
 */
export async function softDeleteLabel(
  projectId: string,
  labelId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.delete(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/labels/${encodeURIComponent(labelId)}`
  );
}

/**
 * Get labels for a task.
 */
export async function getTaskLabels(
  projectId: string,
  taskId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Label[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<ListResponse<Label>>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/tasks/${encodeURIComponent(taskId)}/labels`
  );
  return response.data;
}

/**
 * Set labels for a task (replaces all).
 */
export async function setTaskLabels(
  projectId: string,
  taskId: string,
  payload: SetLabelsPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Label[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.put<ListResponse<Label>>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/tasks/${encodeURIComponent(taskId)}/labels`,
    payload
  );
  return response.data;
}
