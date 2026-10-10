/**
 * User commands - user-facing project and task functions
 *
 * These are simpler than admin commands, scoped to the authenticated user's
 * own projects and tasks.
 */
import { getHttpClient } from "../utils/client-factory.js";
import type { PagedListResponse } from "../types/common-types.js";
import { appendPageListParams } from "@inflatable-cookie/underlay/client/page-lists";

const USER_LIST_PAGE_SIZE = 100;

async function collectPages<T>(
  fetchPage: (page: number) => Promise<PagedListResponse<T>>,
): Promise<T[]> {
  const data: T[] = [];
  let page = 1;
  while (true) {
    const response = await fetchPage(page);
    data.push(...response.data);
    if (!response.hasMore) return data;
    page += 1;
  }
}

type NightfireValue = {
  schema: string;
  block?: unknown;
  blocks?: unknown[];
};

// ============================================================================
// Types
// ============================================================================

export interface UserProject {
  id: string;
  name: string;
  description?: NightfireValue | null;
  status: string;
  taskSummary: {
    total: number;
    completed: number;
  };
  createdAt: string;
  updatedAt: string;
}

export interface UserTask {
  id: string;
  projectId: string;
  title: string;
  description?: string | null;
  status: string;
  priority: string;
  dueDate?: string | null;
  completedAt?: string | null;
  position: number;
  createdAt: string;
  updatedAt: string;
}

export interface CreateUserProjectPayload {
  name: string;
  description?: NightfireValue | null;
}

export interface UpdateUserProjectPayload {
  name?: string;
  description?: NightfireValue | null;
  status?: string;
}

export interface CreateUserTaskPayload {
  title: string;
  description?: string | null;
  priority?: string;
  dueDate?: string | null;
}

export interface UpdateUserTaskPayload {
  title?: string;
  description?: string | null;
  status?: string;
  priority?: string;
  dueDate?: string | null;
}

interface SingleResponse<T> {
  data: T;
}

// ============================================================================
// Project Commands
// ============================================================================

export async function listProjects(
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<UserProject[]> {
  const http = getHttpClient({ fetchFn, accessToken });
  return collectPages((page) =>
    http.get<PagedListResponse<UserProject>>(
      appendPageListParams("/v1/projects", { page, limit: USER_LIST_PAGE_SIZE }),
    ),
  );
}

export async function createProject(
  payload: CreateUserProjectPayload,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<UserProject> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<UserProject>>("/v1/projects", payload);
  return response.data;
}

export async function getProject(
  projectId: string,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<UserProject> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<UserProject>>(`/v1/projects/${projectId}`);
  return response.data;
}

export async function updateProject(
  projectId: string,
  payload: UpdateUserProjectPayload,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<UserProject> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.patch<SingleResponse<UserProject>>(`/v1/projects/${projectId}`, payload);
  return response.data;
}

export async function deleteProject(
  projectId: string,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<void> {
  const http = getHttpClient({ fetchFn, accessToken });
  await http.delete<void>(`/v1/projects/${projectId}`);
}

// ============================================================================
// Task Commands
// ============================================================================

export async function listTasks(
  projectId: string,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<UserTask[]> {
  const http = getHttpClient({ fetchFn, accessToken });
  const path = `/v1/projects/${projectId}/tasks`;
  return collectPages((page) =>
    http.get<PagedListResponse<UserTask>>(
      appendPageListParams(path, { page, limit: USER_LIST_PAGE_SIZE }),
    ),
  );
}

export async function createTask(
  projectId: string,
  payload: CreateUserTaskPayload,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<UserTask> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<UserTask>>(`/v1/projects/${projectId}/tasks`, payload);
  return response.data;
}

export async function updateTask(
  projectId: string,
  taskId: string,
  payload: UpdateUserTaskPayload,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<UserTask> {
  const http = getHttpClient({ fetchFn, accessToken });
  const response = await http.patch<SingleResponse<UserTask>>(`/v1/projects/${projectId}/tasks/${taskId}`, payload);
  return response.data;
}

export async function deleteTask(
  projectId: string,
  taskId: string,
  fetchFn: typeof fetch,
  accessToken: string,
): Promise<void> {
  const http = getHttpClient({ fetchFn, accessToken });
  await http.delete<void>(`/v1/projects/${projectId}/tasks/${taskId}`);
}
