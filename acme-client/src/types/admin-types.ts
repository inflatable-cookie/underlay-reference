/**
 * Admin types for Acme API
 *
 * These types match the admin route responses from acme-api.
 */

import type { NightfireValue } from "@decodelabs/underlay/nightfire/validation";

// ============================================================================
// Categories
// ============================================================================

export interface Category {
  id: string;
  name: string;
  slug: string;
  description?: string | null;
  color?: string | null;
  weight: number;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface CategoryWithCounts extends Category {
  projectCount: number;
}

export interface CreateCategoryPayload {
  name: string;
  slug: string;
  description?: string | null;
  color?: string | null;
}

export interface UpdateCategoryPayload {
  name?: string;
  slug?: string;
  description?: string | null;
  color?: string | null;
  isActive?: boolean;
}

// ============================================================================
// Projects
// ============================================================================

export interface Project {
  id: string;
  ownerId: string;
  categoryId?: string | null;
  categoryName?: string | null;
  name: string;
  description?: NightfireValue | null;
  status: string;
  weight: number;
  /** Omitted by some admin project responses until task aggregates exist. */
  taskSummary?: {
    total: number;
    completed: number;
  };
  createdAt: string;
  updatedAt: string;
}

export interface ProjectWithCounts extends Project {
  categoryName?: string | null;
  taskCount: number;
  completedTaskCount: number;
}

export interface CreateProjectPayload {
  name: string;
  description?: NightfireValue | null;
  categoryId?: string | null;
  ownerId?: string | null; // Admin can create for other users
}

export interface UpdateProjectPayload {
  name?: string;
  description?: NightfireValue | null;
  status?: string;
  categoryId?: string | null;
}

// ============================================================================
// Tasks
// ============================================================================

export interface Task {
  id: string;
  projectId: string;
  title: string;
  description?: string | null;
  notes?: NightfireValue | null;
  status: string;
  priority: string;
  dueDate?: string | null;
  completedAt?: string | null;
  position: number;
  weight: number;
  createdAt: string;
  updatedAt: string;
}

export interface TaskWithLabels extends Task {
  labelCount: number;
}

export interface CreateTaskPayload {
  title: string;
  description?: string | null;
  notes?: NightfireValue | null;
  priority?: string;
  dueDate?: string | null;
  labelIds?: string[];
}

export interface UpdateTaskPayload {
  title?: string;
  description?: string | null;
  notes?: NightfireValue | null;
  status?: string;
  priority?: string;
  dueDate?: string | null;
  labelIds?: string[];
}

// ============================================================================
// Labels
// ============================================================================

export interface Label {
  id: string;
  projectId: string;
  name: string;
  color: string;
  weight: number;
  createdAt: string;
}

export interface CreateLabelPayload {
  name: string;
  color?: string;
}

export interface SetLabelsPayload {
  labelIds: string[];
}

// ============================================================================
// Reorder
// ============================================================================

export interface ReorderPayload {
  ids: string[];
}

export interface ReorderResult {
  ok: boolean;
}

// ============================================================================
// Validation
// ============================================================================

export interface ValidateFieldPayload {
  /** Entity type: "category", "project", "label" */
  entity: string;
  /** Field name: "slug", "name" */
  field: string;
  /** Value to validate */
  value: string;
  /** Context value (e.g., project_id for label uniqueness) */
  context_value?: string;
  /** ID to exclude from uniqueness check (for edits) */
  exclude_id?: string;
}

export interface ValidationResult {
  valid: boolean;
  message?: string;
}

// ============================================================================
// Status and Priority constants
// ============================================================================

export const ProjectStatus = {
  Active: "active",
  Archived: "archived",
  OnHold: "on_hold",
} as const;

export type ProjectStatus = (typeof ProjectStatus)[keyof typeof ProjectStatus];

export const TaskStatus = {
  Pending: "pending",
  InProgress: "in_progress",
  Completed: "completed",
} as const;

export type TaskStatus = (typeof TaskStatus)[keyof typeof TaskStatus];

export const TaskPriority = {
  Low: "low",
  Medium: "medium",
  High: "high",
  Urgent: "urgent",
} as const;

export type TaskPriority = (typeof TaskPriority)[keyof typeof TaskPriority];

// ============================================================================
// Users
// ============================================================================

export const UserRole = {
  User: "user",
  Tester: "tester",
  Editor: "editor",
  Admin: "admin",
  Support: "support",
  Superadmin: "superadmin",
} as const;

export type UserRole = (typeof UserRole)[keyof typeof UserRole];

export const UserStatus = {
  Active: "active",
  Suspended: "suspended",
  Deleted: "deleted",
} as const;

export type UserStatus = (typeof UserStatus)[keyof typeof UserStatus];

export interface User {
  id: string;
  email: string;
  role: UserRole;
  status: UserStatus;
  displayName?: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface UserDetail extends User {
  activeSessionCount: number;
  failedLoginCount: number;
  lockoutUntil?: string | null;
}

export interface CreateUserPayload {
  email: string;
  role: UserRole;
  status: UserStatus;
  displayName?: string | null;
  sendPasswordReset?: boolean;
}

export interface UpdateUserPayload {
  email?: string;
  role?: UserRole;
  status?: UserStatus;
  displayName?: string | null;
}

export interface ListUsersQuery {
  sort?: Array<{ field: string; direction: "asc" | "desc" }>;
  filters?: Array<{ field: string; operator?: "eq" | "ne" | "gt" | "gte" | "lt" | "lte" | "like"; value: string }>;
  page?: number;
  limit?: number;
}

export interface UpdateUserRolePayload {
  role: UserRole;
}

export interface UserListResponse {
  data: User[];
  hasMore: boolean;
  total: number;
}

// ============================================================================
// Dashboard
// ============================================================================

export interface UserCounts {
  active: number;
  suspended: number;
  deleted: number;
  total: number;
}

export interface DashboardStats {
  userCounts: UserCounts;
  mediaCount: number;
  recentRegistrations: number;
  activeSessions: number;
}

// ============================================================================
// Activity/Audit Log
// ============================================================================

export interface ActivityActor {
  id: string;
  email: string;
  displayName?: string | null;
}

export interface ActivityEntry {
  id: string;
  occurredAt: string;
  actor?: ActivityActor | null;
  action: string;
  resourceType: string;
  resourceId: string;
  details: Record<string, unknown>;
}

export interface ActivityListResponse {
  data: ActivityEntry[];
  hasMore: boolean;
  total: number;
}

export interface ListActivityQuery {
  action?: string;
  resourceType?: string;
  page?: number;
  limit?: number;
  offset?: number;
}

// ============================================================================
// Batch Operations
// ============================================================================

export interface BatchDeletePayload {
  ids: string[];
}

export interface BatchDeleteResult {
  ok: boolean;
  deleted: number;
}

export interface BatchUpdateTaskStatusPayload {
  ids: string[];
  status: TaskStatus;
}

export interface BatchUpdateResult {
  ok: boolean;
  updated: number;
}

// ============================================================================
// Jobs
// ============================================================================

export type JobStatus = "pending" | "claimed" | "running" | "succeeded" | "failed" | "cancelled";

export interface JobProgress {
  current: number;
  total: number;
  percent: number;
  message?: string | null;
}

export interface JobSummary {
  id: string;
  jobType: string;
  status: JobStatus;
  attempts: number;
  maxAttempts: number;
  createdAt: string;
  startedAt?: string | null;
  finishedAt?: string | null;
  errorMessage?: string | null;
}

export interface JobDetail extends JobSummary {
  payload: Record<string, unknown>;
  progress?: JobProgress | null;
  scheduledFor?: string | null;
}

export interface JobStats {
  pending: number;
  running: number;
  failed: number;
  succeeded: number;
}

export interface ListJobsQuery {
  status?: JobStatus;
  jobType?: string;
  page?: number;
  limit?: number;
}

export interface JobListResponse {
  data: JobSummary[];
  hasMore: boolean;
  total: number;
}

// ============================================================================
// Scheduled Tasks
// ============================================================================

export interface ScheduledTaskSummary {
  id: string;
  name: string;
  jobType: string;
  schedule: string;
  enabled: boolean;
  lastScheduledAt?: string | null;
  lastCompletedAt?: string | null;
  createdAt: string;
}

export interface ScheduledTaskDetail {
  id: string;
  name: string;
  jobType: string;
  schedule: string;
  payload: unknown;
  maxAttempts: number;
  timeoutSeconds?: number | null;
  allowOverlap: boolean;
  priority: number;
  enabled: boolean;
  lastScheduledAt?: string | null;
  lastCompletedAt?: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface ListScheduledTasksQuery {
  enabled?: boolean;
  page?: number;
  limit?: number;
}

export interface ScheduledTaskListResponse {
  data: ScheduledTaskSummary[];
  hasMore: boolean;
  total: number;
}

export interface TriggerScheduledTaskResult {
  jobId: string;
}

// ============================================================================
// Error Logs
// ============================================================================

export interface ErrorLogSummary {
  id: string;
  occurredAt: string;
  endpoint: string;
  method: string;
  statusCode: number;
  errorCode: string;
  message: string;
  correlationId: string;
}

export interface ErrorLogDetail extends ErrorLogSummary {
  context: Record<string, unknown>;
}

export interface ErrorLogStats {
  totalLast24h: number;
  serverErrorsLast24h: number;
  clientErrorsLast24h: number;
}

export interface ListErrorLogsQuery {
  status_class?: "4xx" | "5xx";
  status_code?: number;
  error_code?: string;
  endpoint?: string;
  since?: string;
  until?: string;
  limit?: number;
  offset?: number;
}

export interface ErrorLogsListResponse {
  data: ErrorLogSummary[];
  total: number;
  hasMore: boolean;
}
