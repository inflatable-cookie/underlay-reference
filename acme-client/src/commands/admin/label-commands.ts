import type { ListResponse, SingleResponse } from "../../types/common-types.js";
import type {
  Label,
  CreateLabelPayload,
  SetLabelsPayload,
} from "../../types/admin-types.js";
import { getAdminHttpClient } from "../../utils/client-factory.js";

/**
 * List labels for a project.
 */
export async function listLabels(
  projectId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Label[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<ListResponse<Label>>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/labels`
  );
  return response.data;
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
