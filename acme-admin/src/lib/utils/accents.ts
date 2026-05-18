/**
 * Shared accent color mappings for retained Underlay surfaces and
 * semantic tone mappings for Poodle Pill replacements.
 *
 * Each entity domain keeps its own mapping helpers so replacements stay
 * explicit instead of relying on one generic color-to-tone translation.
 */

export type PillTone = "neutral" | "success" | "danger";

export const ACCENT_COLORS = {
  red: "#dc2626",
  redSoft: "#ef4444",
  violet: "#8b5cf6",
  indigo: "#6366f1",
  blue: "#3b82f6",
  teal: "#14b8a6",
  green: "#22c55e",
  emerald: "#10b981",
  orange: "#f97316",
  amber: "#f59e0b",
  slate: "#64748b",
  gray: "#6b7280"
} as const;

// ============================================================================
// Users
// ============================================================================

export function getUserRoleAccent(role: string): string {
  switch (role) {
    case "superadmin": return ACCENT_COLORS.red;
    case "admin": return ACCENT_COLORS.violet;
    case "support": return ACCENT_COLORS.blue;
    case "editor": return ACCENT_COLORS.teal;
    case "tutor": return ACCENT_COLORS.green;
    case "tester": return ACCENT_COLORS.orange;
    default: return ACCENT_COLORS.slate;
  }
}

export function getUserRoleTone(role: string): PillTone {
  return role === "superadmin" ? "danger" : "neutral";
}

export function getUserStatusAccent(status: string): string {
  switch (status) {
    case "active": return ACCENT_COLORS.green;
    case "suspended": return ACCENT_COLORS.orange;
    case "deleted": return ACCENT_COLORS.red;
    default: return ACCENT_COLORS.slate;
  }
}

export function getUserStatusTone(status: string): PillTone {
  switch (status) {
    case "active": return "success";
    case "deleted": return "danger";
    default: return "neutral";
  }
}

export function getSessionStatusAccent(status: string): string {
  switch (status) {
    case "active": return ACCENT_COLORS.green;
    case "expired": return ACCENT_COLORS.amber;
    case "revoked": return ACCENT_COLORS.red;
    default: return ACCENT_COLORS.slate;
  }
}

export function getSessionStatusTone(status: string): PillTone {
  switch (status) {
    case "active": return "success";
    case "revoked": return "danger";
    default: return "neutral";
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
      return ACCENT_COLORS.green;
    case "delete":
    case "deleted":
    case "soft_delete":
    case "suspend":
      return ACCENT_COLORS.red;
    case "update":
    case "updated":
    case "upload":
    case "uploaded":
    case "role_change":
      return ACCENT_COLORS.blue;
    case "login":
    case "logout":
      return ACCENT_COLORS.slate;
    default:
      return ACCENT_COLORS.violet;
  }
}

export function getActivityTone(action: string): PillTone {
  switch (action) {
    case "create":
    case "created":
    case "restore":
    case "restored":
    case "unsuspend":
      return "success";
    case "delete":
    case "deleted":
    case "soft_delete":
    case "suspend":
      return "danger";
    default:
      return "neutral";
  }
}

// ============================================================================
// Projects
// ============================================================================

export function getProjectStatusAccent(status: string): string {
  switch (status) {
    case "active": return ACCENT_COLORS.emerald;
    case "archived": return ACCENT_COLORS.gray;
    case "on_hold": return ACCENT_COLORS.amber;
    default: return ACCENT_COLORS.slate;
  }
}

export function getProjectStatusTone(status: string): PillTone {
  return status === "active" ? "success" : "neutral";
}

// ============================================================================
// Tasks
// ============================================================================

export function getTaskStatusAccent(status: string): string {
  switch (status) {
    case "pending": return ACCENT_COLORS.gray;
    case "in_progress": return ACCENT_COLORS.blue;
    case "completed": return ACCENT_COLORS.emerald;
    case "cancelled": return ACCENT_COLORS.redSoft;
    default: return ACCENT_COLORS.slate;
  }
}

export function getTaskStatusTone(status: string): PillTone {
  switch (status) {
    case "completed": return "success";
    case "cancelled": return "danger";
    default: return "neutral";
  }
}

export function getTaskPriorityAccent(priority: string): string {
  switch (priority) {
    case "low": return ACCENT_COLORS.gray;
    case "medium": return ACCENT_COLORS.slate;
    case "high": return ACCENT_COLORS.amber;
    case "urgent": return ACCENT_COLORS.redSoft;
    default: return ACCENT_COLORS.slate;
  }
}

export function getTaskPriorityTone(priority: string): PillTone {
  return priority === "urgent" ? "danger" : "neutral";
}

// ============================================================================
// Jobs
// ============================================================================

export function getJobStatusAccent(status: string): string {
  switch (status) {
    case "succeeded": return ACCENT_COLORS.emerald;
    case "failed": return ACCENT_COLORS.redSoft;
    case "running": return ACCENT_COLORS.blue;
    case "pending": return ACCENT_COLORS.amber;
    case "cancelled": return ACCENT_COLORS.gray;
    default: return ACCENT_COLORS.slate;
  }
}

export function getJobStatusTone(status: string): PillTone {
  switch (status) {
    case "succeeded": return "success";
    case "failed": return "danger";
    default: return "neutral";
  }
}

export function getSystemCardAccent(card: string): string {
  switch (card) {
    case "errors": return ACCENT_COLORS.red;
    case "jobs": return ACCENT_COLORS.violet;
    case "scheduled_tasks": return ACCENT_COLORS.emerald;
    case "audit": return ACCENT_COLORS.indigo;
    default: return ACCENT_COLORS.slate;
  }
}

export function getMediaVisibilityPillAccent(visibility: string): string {
  return visibility === "restricted" ? ACCENT_COLORS.amber : ACCENT_COLORS.blue;
}

export function getMediaMetaAccent(kind: "deleted" | "current" | "usage"): string {
  switch (kind) {
    case "deleted": return ACCENT_COLORS.redSoft;
    case "current": return ACCENT_COLORS.blue;
    case "usage": return ACCENT_COLORS.indigo;
    default: return ACCENT_COLORS.slate;
  }
}

export function getMediaMetaTone(kind: "deleted" | "current" | "usage"): PillTone {
  switch (kind) {
    case "deleted": return "danger";
    case "current": return "success";
    default: return "neutral";
  }
}
