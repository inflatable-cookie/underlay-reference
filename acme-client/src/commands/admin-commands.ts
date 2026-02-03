/**
 * Admin commands for Acme API
 *
 * Provides CRUD operations for admin routes with filtering/sorting support.
 */
import type { ListResponse, SingleResponse } from "../types/common-types.js";
import type {
  Category,
  CategoryWithCounts,
  CreateCategoryPayload,
  UpdateCategoryPayload,
  Project,
  ProjectWithCounts,
  CreateProjectPayload,
  UpdateProjectPayload,
  Task,
  TaskWithLabels,
  CreateTaskPayload,
  UpdateTaskPayload,
  Label,
  CreateLabelPayload,
  SetLabelsPayload,
  ReorderPayload,
  ReorderResult,
  ValidateFieldPayload,
  ValidationResult,
  User,
  UserDetail,
  ListUsersQuery,
  UpdateUserRolePayload,
  UserListResponse,
  DashboardStats,
  ActivityListResponse,
  ListActivityQuery,
  BatchDeletePayload,
  BatchDeleteResult,
  BatchUpdateTaskStatusPayload,
  BatchUpdateResult,
} from "../types/admin-types.js";
import { getAdminHttpClient } from "../utils/client-factory.js";
import {
  appendQueryParams,
  type QueryParams,
} from "@decodelabs/underlay/client";
import {
  appendSuggestionParams,
  type SuggestionRequestOptions,
} from "@decodelabs/underlay/patterns";

// ============================================================================
// Categories
// ============================================================================

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
  const path = appendQueryParams("/v1/admin/categories", query ?? {});
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
 * Restore a soft-deleted category.
 */
export async function restoreCategory(
  categoryId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Category> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<Category>>(
    `/v1/admin/categories/${encodeURIComponent(categoryId)}/restore`,
    {}
  );
  return response.data;
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

// ============================================================================
// Projects
// ============================================================================

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
  const path = appendQueryParams("/v1/admin/projects", query ?? {});
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

// ============================================================================
// Tasks
// ============================================================================

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
    query ?? {}
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
    `/v1/admin/projects/${encodeURIComponent(projectId)}/tasks:batch-delete`,
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
    `/v1/admin/projects/${encodeURIComponent(projectId)}/tasks:batch-update`,
    payload
  );
}

// ============================================================================
// Labels
// ============================================================================

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

// ============================================================================
// Validation
// ============================================================================

/**
 * Validate a field value (async form validation).
 *
 * Used for checking uniqueness of slugs, names, etc. before form submission.
 *
 * @example
 * ```typescript
 * const result = await validateField(
 *   { entity: 'category', field: 'slug', value: 'my-category' },
 *   fetch,
 *   accessToken
 * );
 * if (!result.valid) {
 *   showError(result.message);
 * }
 * ```
 */
export async function validateField(
  payload: ValidateFieldPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<ValidationResult> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  return await http.post<ValidationResult>(
    "/v1/admin/validate-field",
    payload
  );
}

// ============================================================================
// Dashboard
// ============================================================================

/**
 * Get dashboard statistics.
 */
export async function getDashboardStats(
  fetchFn: typeof fetch,
  accessToken: string
): Promise<DashboardStats> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<DashboardStats>>(
    "/v1/admin/dashboard/stats"
  );
  return response.data;
}

// ============================================================================
// Users
// ============================================================================

/**
 * List users (admin).
 *
 * Supports filtering by role, status, and email search.
 */
export async function listUsers(
  fetchFn: typeof fetch,
  accessToken: string,
  query?: ListUsersQuery
): Promise<UserListResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });

  // Build query params
  const params = new URLSearchParams();
  if (query?.role) params.set("role", query.role);
  if (query?.status) params.set("status", query.status);
  if (query?.search) params.set("search", query.search);
  if (query?.limit !== undefined) params.set("limit", String(query.limit));
  if (query?.offset !== undefined) params.set("offset", String(query.offset));

  const queryString = params.toString();
  const path = queryString ? `/v1/admin/users?${queryString}` : "/v1/admin/users";

  return await http.get<UserListResponse>(path);
}

/**
 * Get a single user by ID.
 */
export async function getUser(
  userId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<UserDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<UserDetail>>(
    `/v1/admin/users/${encodeURIComponent(userId)}`
  );
  return response.data;
}

/**
 * Update a user's role.
 */
export async function updateUserRole(
  userId: string,
  payload: UpdateUserRolePayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<User> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.put<SingleResponse<User>>(
    `/v1/admin/users/${encodeURIComponent(userId)}/role`,
    payload
  );
  return response.data;
}

/**
 * Suspend a user.
 */
export async function suspendUser(
  userId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<User> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<User>>(
    `/v1/admin/users/${encodeURIComponent(userId)}/suspend`,
    {}
  );
  return response.data;
}

/**
 * Unsuspend (reactivate) a user.
 */
export async function unsuspendUser(
  userId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<User> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<User>>(
    `/v1/admin/users/${encodeURIComponent(userId)}/unsuspend`,
    {}
  );
  return response.data;
}

// ============================================================================
// Activity/Audit Log
// ============================================================================

/**
 * List all activity (global feed).
 */
export async function listActivity(
  fetchFn: typeof fetch,
  accessToken: string,
  query?: ListActivityQuery
): Promise<ActivityListResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });

  const params = new URLSearchParams();
  if (query?.limit !== undefined) params.set("limit", String(query.limit));
  if (query?.offset !== undefined) params.set("offset", String(query.offset));

  const queryString = params.toString();
  const path = queryString ? `/v1/admin/activity?${queryString}` : "/v1/admin/activity";

  return await http.get<ActivityListResponse>(path);
}

/**
 * List activity for a specific entity.
 */
export async function listActivityForEntity(
  entityType: string,
  entityId: string,
  fetchFn: typeof fetch,
  accessToken: string,
  query?: ListActivityQuery
): Promise<ActivityListResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });

  const params = new URLSearchParams();
  if (query?.limit !== undefined) params.set("limit", String(query.limit));
  if (query?.offset !== undefined) params.set("offset", String(query.offset));

  const queryString = params.toString();
  const basePath = `/v1/admin/activity/entity/${encodeURIComponent(entityType)}/${encodeURIComponent(entityId)}`;
  const path = queryString ? `${basePath}?${queryString}` : basePath;

  return await http.get<ActivityListResponse>(path);
}

/**
 * List activity performed by a specific user.
 */
export async function listActivityForUser(
  userId: string,
  fetchFn: typeof fetch,
  accessToken: string,
  query?: ListActivityQuery
): Promise<ActivityListResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });

  const params = new URLSearchParams();
  if (query?.limit !== undefined) params.set("limit", String(query.limit));
  if (query?.offset !== undefined) params.set("offset", String(query.offset));

  const queryString = params.toString();
  const basePath = `/v1/admin/users/${encodeURIComponent(userId)}/activity`;
  const path = queryString ? `${basePath}?${queryString}` : basePath;

  return await http.get<ActivityListResponse>(path);
}
