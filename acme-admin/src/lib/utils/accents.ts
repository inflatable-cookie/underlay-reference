/**
 * Shared accent color mappings for Pill/Badge components.
 *
 * Each entity domain has its own function to keep the mappings
 * explicit and avoid collisions between status values that mean
 * different things in different contexts.
 */

// ============================================================================
// Users
// ============================================================================

export function getUserRoleAccent(role: string): string {
  switch (role) {
    case "superadmin": return "#dc2626";
    case "admin": return "#8b5cf6";
    case "support": return "#3b82f6";
    case "editor": return "#14b8a6";
    case "tutor": return "#22c55e";
    case "tester": return "#f97316";
    default: return "#64748b";
  }
}

export function getUserStatusAccent(status: string): string {
  switch (status) {
    case "active": return "#22c55e";
    case "suspended": return "#f97316";
    case "deleted": return "#dc2626";
    default: return "#64748b";
  }
}

export function getSessionStatusAccent(status: string): string {
  switch (status) {
    case "active": return "#22c55e";
    case "expired": return "#f59e0b";
    case "revoked": return "#dc2626";
    default: return "#64748b";
  }
}

// ============================================================================
// Activity / Audit
// ============================================================================

export function getActivityAccent(action: string): string {
  switch (action) {
    case "create":
    case "created":
    case "restore":
    case "restored":
    case "unsuspend":
      return "#22c55e";
    case "delete":
    case "deleted":
    case "soft_delete":
    case "suspend":
      return "#dc2626";
    case "update":
    case "updated":
    case "upload":
    case "uploaded":
    case "role_change":
      return "#3b82f6";
    case "login":
    case "logout":
      return "#64748b";
    default:
      return "#8b5cf6";
  }
}

// ============================================================================
// Projects
// ============================================================================

export function getProjectStatusAccent(status: string): string {
  switch (status) {
    case "active": return "#10b981";
    case "archived": return "#6b7280";
    case "on_hold": return "#f59e0b";
    default: return "#64748b";
  }
}

// ============================================================================
// Tasks
// ============================================================================

export function getTaskStatusAccent(status: string): string {
  switch (status) {
    case "pending": return "#6b7280";
    case "in_progress": return "#3b82f6";
    case "completed": return "#10b981";
    case "cancelled": return "#ef4444";
    default: return "#64748b";
  }
}

export function getTaskPriorityAccent(priority: string): string {
  switch (priority) {
    case "low": return "#6b7280";
    case "medium": return "#64748b";
    case "high": return "#f59e0b";
    case "urgent": return "#ef4444";
    default: return "#64748b";
  }
}

// ============================================================================
// Jobs
// ============================================================================

export function getJobStatusAccent(status: string): string {
  switch (status) {
    case "succeeded": return "#10b981";
    case "failed": return "#ef4444";
    case "running": return "#3b82f6";
    case "pending": return "#f59e0b";
    case "cancelled": return "#6b7280";
    default: return "#64748b";
  }
}
