/**
 * User commands - user-facing project and task functions
 *
 * These are simpler than admin commands, scoped to the authenticated user's
 * own projects and tasks.
 */
import { getHttpClient } from "../utils/client-factory.js";

// ============================================================================
// Types
// ============================================================================

export interface UserProject {
  id: string;
  name: string;
  description?: string | null;
  status: string;
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
  description?: string | null;
}

export interface UpdateUserProjectPayload {
  name?: string;
  description?: string | null;
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

interface ItemsResponse<T> {
  items: T[];
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
  const response = await http.get<ItemsResponse<UserProject>>("/v1/projects");
  return response.items;
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
  const response = await http.get<ItemsResponse<UserTask>>(`/v1/projects/${projectId}/tasks`);
  return response.items;
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
