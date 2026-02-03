/**
 * Admin types for Acme API
 *
 * These types match the admin route responses from acme-api.
 */

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
  name: string;
  description?: string | null;
  status: string;
  weight: number;
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
  description?: string | null;
  categoryId?: string | null;
  ownerId?: string | null; // Admin can create for other users
}

export interface UpdateProjectPayload {
  name?: string;
  description?: string | null;
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
  priority?: string;
  dueDate?: string | null;
  labelIds?: string[];
}

export interface UpdateTaskPayload {
  title?: string;
  description?: string | null;
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
  contextValue?: string;
  /** ID to exclude from uniqueness check (for edits) */
  excludeId?: string;
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
