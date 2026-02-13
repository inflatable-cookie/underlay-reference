import type { ListResponse, SingleResponse } from "../../types/common-types.js";
import type {
  Task,
  TaskWithLabels,
  CreateTaskPayload,
  UpdateTaskPayload,
  ReorderPayload,
  ReorderResult,
  BatchDeletePayload,
  BatchDeleteResult,
  BatchUpdateTaskStatusPayload,
  BatchUpdateResult,
} from "../../types/admin-types.js";
import { getAdminHttpClient } from "../../utils/client-factory.js";
import {
  appendQueryParams,
  type QueryParams,
} from "@decodelabs/underlay/client";
import { toSnakeQueryParams } from "./utils.js";

/**
 * List tasks for a project (admin).
 *
 * Supports filtering and sorting via QueryParams:
 * - `sort=position:asc,dueDate:asc`
 * - `filter[status]=pending`
 * - `filter[priority]=high`
 */
export async function listTasks(
  projectId: string,
  fetchFn: typeof fetch,
  accessToken: string,
  query?: QueryParams
): Promise<TaskWithLabels[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const path = appendQueryParams(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/tasks`,
    toSnakeQueryParams(query)
  );
  const response = await http.get<ListResponse<TaskWithLabels>>(path);
  return response.data;
}

/**
 * Get a single task by ID.
 */
export async function getTask(
  projectId: string,
  taskId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Task> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<Task>>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/tasks/${encodeURIComponent(taskId)}`
  );
  return response.data;
}

/**
 * Create a new task.
 */
export async function createTask(
  projectId: string,
  payload: CreateTaskPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Task> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<Task>>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/tasks`,
    payload
  );
  return response.data;
}

/**
 * Update an existing task.
 */
export async function updateTask(
  projectId: string,
  taskId: string,
  payload: UpdateTaskPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Task> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.patch<SingleResponse<Task>>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/tasks/${encodeURIComponent(taskId)}`,
    payload
  );
  return response.data;
}

/**
 * Soft delete a task.
 */
export async function softDeleteTask(
  projectId: string,
  taskId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.delete(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/tasks/${encodeURIComponent(taskId)}`
  );
}

/**
 * Reorder tasks within a project.
 */
export async function reorderTasks(
  projectId: string,
  payload: ReorderPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<ReorderResult> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  return await http.put<ReorderResult>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/tasks/reorder`,
    payload
  );
}

/**
 * Batch delete tasks.
 */
export async function batchDeleteTasks(
  projectId: string,
  payload: BatchDeletePayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<BatchDeleteResult> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  return await http.post<BatchDeleteResult>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/tasks/batch-delete`,
    payload
  );
}

/**
 * Batch update task status.
 */
export async function batchUpdateTaskStatus(
  projectId: string,
  payload: BatchUpdateTaskStatusPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<BatchUpdateResult> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  return await http.post<BatchUpdateResult>(
    `/v1/admin/projects/${encodeURIComponent(projectId)}/tasks/batch-update`,
    payload
  );
}
