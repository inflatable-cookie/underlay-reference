/**
 * Admin commands for Acme API
 *
 * Provides CRUD operations for admin routes with filtering/sorting support.
 */
import type { ListResponse, SingleResponse, Session } from "../types/common-types.js";
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
  CreateUserPayload,
  ListUsersQuery,
  UpdateUserPayload,
  UpdateUserRolePayload,
  UserListResponse,
  DashboardStats,
  ActivityListResponse,
  ListActivityQuery,
  BatchDeletePayload,
  BatchDeleteResult,
  BatchUpdateTaskStatusPayload,
  BatchUpdateResult,
  JobSummary,
  JobDetail,
  JobStats,
  ListJobsQuery,
  ScheduledTaskSummary,
  ScheduledTaskDetail,
  ListScheduledTasksQuery,
  TriggerScheduledTaskResult,
  CapturedEmailSummary,
  CapturedEmailDetail,
  ListCapturedEmailsQuery,
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

function camelToSnake(value: string): string {
  return value.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toLowerCase();
}

function toSnakeQueryParams(query?: QueryParams): QueryParams {
  if (!query) {
    return {};
  }

  return {
    ...query,
    sort: query.sort?.map((item) => ({
      ...item,
      field: camelToSnake(item.field),
    })),
    filters: query.filters?.map((item) => ({
      ...item,
      field: camelToSnake(item.field),
    })),
  };
}

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
 * Create a user (admin).
 *
 * Creates a user record without logging the admin in as that user.
 * Optionally sends a password reset email so the user can set an initial password.
 */
export async function createUser(
  payload: CreateUserPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<User> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<User>>("/v1/admin/users", {
    email: payload.email,
    role: payload.role,
    status: payload.status,
    displayName: payload.displayName ?? null,
    sendPasswordReset: payload.sendPasswordReset ?? true,
  });
  return response.data;
}

/**
 * List users (admin).
 *
 * Supports filtering by role, status, and text search.
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
  if (query?.displayName) params.set("display_name", query.displayName);
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
 * Update a user (admin).
 */
export async function updateUser(
  userId: string,
  payload: UpdateUserPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<User> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.put<SingleResponse<User>>(
    `/v1/admin/users/${encodeURIComponent(userId)}`,
    {
      email: payload.email,
      role: payload.role,
      status: payload.status,
      displayName: payload.displayName ?? null,
    }
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

/**
 * List all sessions for a user (admin).
 *
 * Returns all sessions (active, expired, revoked) for administrative purposes.
 */
export async function listUserSessions(
  userId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<Session[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<ListResponse<Session>>(
    `/v1/admin/users/${encodeURIComponent(userId)}/sessions`
  );
  return response.data;
}

/**
 * Revoke a specific session for a user (admin).
 *
 * Terminates the session and logs out the user from that device.
 */
export async function revokeUserSession(
  userId: string,
  sessionId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.post(
    `/v1/admin/users/${encodeURIComponent(userId)}/sessions/${encodeURIComponent(sessionId)}/revoke`,
    {}
  );
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

// ============================================================================
// Jobs
// ============================================================================

/**
 * List background jobs.
 *
 * Supports filtering by status and job type.
 */
export async function listJobs(
  fetchFn: typeof fetch,
  accessToken: string,
  query?: ListJobsQuery
): Promise<JobSummary[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });

  const params = new URLSearchParams();
  if (query?.status) params.set("status", query.status);
  if (query?.jobType) params.set("job_type", query.jobType);
  if (query?.limit !== undefined) params.set("limit", String(query.limit));

  const queryString = params.toString();
  const path = queryString ? `/v1/admin/jobs?${queryString}` : "/v1/admin/jobs";

  const response = await http.get<ListResponse<JobSummary>>(path);
  return response.data;
}

/**
 * Get details of a specific job.
 */
export async function getJob(
  jobId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<JobDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<JobDetail>>(
    `/v1/admin/jobs/${encodeURIComponent(jobId)}`
  );
  return response.data;
}

/**
 * Get job queue statistics.
 */
export async function getJobStats(
  fetchFn: typeof fetch,
  accessToken: string
): Promise<JobStats> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<JobStats>>(
    "/v1/admin/jobs/stats"
  );
  return response.data;
}

/**
 * Cancel a pending or running job.
 */
export async function cancelJob(
  jobId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<JobDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<JobDetail>>(
    `/v1/admin/jobs/${encodeURIComponent(jobId)}/cancel`,
    {}
  );
  return response.data;
}

/**
 * Retry a failed or cancelled job.
 *
 * Creates a new job with the same payload.
 */
export async function retryJob(
  jobId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<JobDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<JobDetail>>(
    `/v1/admin/jobs/${encodeURIComponent(jobId)}/retry`,
    {}
  );
  return response.data;
}

// ============================================================================
// Scheduled Tasks
// ============================================================================

export async function listScheduledTasks(
  fetchFn: typeof fetch,
  accessToken: string,
  query?: ListScheduledTasksQuery
): Promise<ScheduledTaskSummary[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });

  const params = new URLSearchParams();
  if (query?.enabled !== undefined) params.set("enabled", String(query.enabled));
  if (query?.limit !== undefined) params.set("limit", String(query.limit));
  if (query?.offset !== undefined) params.set("offset", String(query.offset));

  const queryString = params.toString();
  const path = queryString ? `/v1/admin/scheduled-tasks?${queryString}` : "/v1/admin/scheduled-tasks";

  const response = await http.get<ListResponse<ScheduledTaskSummary>>(path);
  return response.data;
}

export async function getScheduledTask(
  taskId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<ScheduledTaskDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<ScheduledTaskDetail>>(
    `/v1/admin/scheduled-tasks/${encodeURIComponent(taskId)}`
  );
  return response.data;
}

export async function toggleScheduledTask(
  taskId: string,
  enabled: boolean,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.post(
    `/v1/admin/scheduled-tasks/${encodeURIComponent(taskId)}/toggle`,
    { enabled }
  );
}

export async function triggerScheduledTask(
  taskId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<TriggerScheduledTaskResult> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<TriggerScheduledTaskResult>>(
    `/v1/admin/scheduled-tasks/${encodeURIComponent(taskId)}/trigger`,
    {}
  );
  return response.data;
}

// ============================================================================
// Captured Emails
// ============================================================================

export async function listCapturedEmails(
  fetchFn: typeof fetch,
  accessToken: string,
  query?: ListCapturedEmailsQuery
): Promise<CapturedEmailSummary[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });

  const params = new URLSearchParams();
  if (query?.to_address) params.set("to_address", query.to_address);
  if (query?.from_address) params.set("from_address", query.from_address);
  if (query?.since) params.set("since", query.since);
  if (query?.until) params.set("until", query.until);
  if (query?.limit !== undefined) params.set("limit", String(query.limit));
  if (query?.offset !== undefined) params.set("offset", String(query.offset));

  const queryString = params.toString();
  const path = queryString ? `/v1/admin/captured-emails?${queryString}` : "/v1/admin/captured-emails";

  const response = await http.get<ListResponse<CapturedEmailSummary>>(path);
  return response.data;
}

export async function getCapturedEmail(
  emailId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<CapturedEmailDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<CapturedEmailDetail>>(
    `/v1/admin/captured-emails/${encodeURIComponent(emailId)}`
  );
  return response.data;
}

export async function deleteCapturedEmail(
  emailId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.delete(`/v1/admin/captured-emails/${encodeURIComponent(emailId)}`);
}

// ============================================================================
// Error Logs
// ============================================================================

import type {
  ErrorLogSummary,
  ErrorLogDetail,
  ErrorLogStats,
  ListErrorLogsQuery,
  ErrorLogsListResponse,
} from "../types/admin-types.js";

/**
 * List error logs with optional filters.
 *
 * Supports filtering by status code, error code, endpoint, and time range.
 */
export async function listErrorLogs(
  fetchFn: typeof fetch,
  accessToken: string,
  query?: ListErrorLogsQuery
): Promise<ErrorLogsListResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });

  const params = new URLSearchParams();
  if (query?.status_code !== undefined) params.set("status_code", String(query.status_code));
  if (query?.error_code) params.set("error_code", query.error_code);
  if (query?.endpoint) params.set("endpoint", query.endpoint);
  if (query?.since) params.set("since", query.since);
  if (query?.until) params.set("until", query.until);
  if (query?.limit !== undefined) params.set("limit", String(query.limit));
  if (query?.offset !== undefined) params.set("offset", String(query.offset));

  const queryString = params.toString();
  const path = queryString ? `/v1/admin/error-logs?${queryString}` : "/v1/admin/error-logs";

  return await http.get<ErrorLogsListResponse>(path);
}

/**
 * Get details of a specific error log entry.
 */
export async function getErrorLog(
  errorLogId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<ErrorLogDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<ErrorLogDetail>>(
    `/v1/admin/error-logs/${encodeURIComponent(errorLogId)}`
  );
  return response.data;
}

/**
 * Get error log statistics.
 */
export async function getErrorLogStats(
  fetchFn: typeof fetch,
  accessToken: string
): Promise<ErrorLogStats> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<ErrorLogStats>>(
    "/v1/admin/error-logs/stats"
  );
  return response.data;
}
