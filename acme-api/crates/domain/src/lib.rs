//! Domain types for the Acme application.
//!
//! This module contains domain entities that represent the core business
//! concepts. Keep HTTP and SQL concerns out of this code.
//!
//! Example domain: A simple task management system with projects and tasks.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// A project that contains tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: uuid::Uuid,
    pub owner_id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: ProjectStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Project status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Active,
    Archived,
}

impl ProjectStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectStatus::Active => "active",
            ProjectStatus::Archived => "archived",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "active" => Some(ProjectStatus::Active),
            "archived" => Some(ProjectStatus::Archived),
            _ => None,
        }
    }
}

/// A task within a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub due_date: Option<NaiveDate>,
    pub completed_at: Option<DateTime<Utc>>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Task completion status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Completed => "completed",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(TaskStatus::Pending),
            "in_progress" => Some(TaskStatus::InProgress),
            "completed" => Some(TaskStatus::Completed),
            "cancelled" => Some(TaskStatus::Cancelled),
            _ => None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskStatus::Completed | TaskStatus::Cancelled)
    }
}

/// Task priority level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
}

impl TaskPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::Low => "low",
            TaskPriority::Medium => "medium",
            TaskPriority::High => "high",
            TaskPriority::Urgent => "urgent",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "low" => Some(TaskPriority::Low),
            "medium" => Some(TaskPriority::Medium),
            "high" => Some(TaskPriority::High),
            "urgent" => Some(TaskPriority::Urgent),
            _ => None,
        }
    }
}

/// A comment on a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComment {
    pub id: uuid::Uuid,
    pub task_id: uuid::Uuid,
    pub author_id: uuid::Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A tag for organizing tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub name: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // ProjectStatus tests
    // ============================================================================

    #[test]
    fn project_status_as_str_returns_correct_strings() {
        assert_eq!(ProjectStatus::Active.as_str(), "active");
        assert_eq!(ProjectStatus::Archived.as_str(), "archived");
    }

    #[test]
    fn project_status_parse_valid_values() {
        assert_eq!(ProjectStatus::parse("active"), Some(ProjectStatus::Active));
        assert_eq!(
            ProjectStatus::parse("archived"),
            Some(ProjectStatus::Archived)
        );
    }

    #[test]
    fn project_status_parse_invalid_values() {
        assert_eq!(ProjectStatus::parse("unknown"), None);
        assert_eq!(ProjectStatus::parse("Active"), None); // case sensitive
        assert_eq!(ProjectStatus::parse(""), None);
    }

    #[test]
    fn project_status_roundtrip() {
        for status in [ProjectStatus::Active, ProjectStatus::Archived] {
            let s = status.as_str();
            let parsed = ProjectStatus::parse(s).expect("should parse own output");
            assert_eq!(parsed, status);
        }
    }

    // ============================================================================
    // TaskStatus tests
    // ============================================================================

    #[test]
    fn task_status_as_str_returns_correct_strings() {
        assert_eq!(TaskStatus::Pending.as_str(), "pending");
        assert_eq!(TaskStatus::InProgress.as_str(), "in_progress");
        assert_eq!(TaskStatus::Completed.as_str(), "completed");
        assert_eq!(TaskStatus::Cancelled.as_str(), "cancelled");
    }

    #[test]
    fn task_status_parse_valid_values() {
        assert_eq!(TaskStatus::parse("pending"), Some(TaskStatus::Pending));
        assert_eq!(TaskStatus::parse("in_progress"), Some(TaskStatus::InProgress));
        assert_eq!(TaskStatus::parse("completed"), Some(TaskStatus::Completed));
        assert_eq!(TaskStatus::parse("cancelled"), Some(TaskStatus::Cancelled));
    }

    #[test]
    fn task_status_parse_invalid_values() {
        assert_eq!(TaskStatus::parse("unknown"), None);
        assert_eq!(TaskStatus::parse("Pending"), None); // case sensitive
        assert_eq!(TaskStatus::parse("in-progress"), None); // wrong separator
    }

    #[test]
    fn task_status_is_terminal() {
        // Terminal statuses
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Cancelled.is_terminal());

        // Non-terminal statuses
        assert!(!TaskStatus::Pending.is_terminal());
        assert!(!TaskStatus::InProgress.is_terminal());
    }

    #[test]
    fn task_status_roundtrip() {
        for status in [
            TaskStatus::Pending,
            TaskStatus::InProgress,
            TaskStatus::Completed,
            TaskStatus::Cancelled,
        ] {
            let s = status.as_str();
            let parsed = TaskStatus::parse(s).expect("should parse own output");
            assert_eq!(parsed, status);
        }
    }

    // ============================================================================
    // TaskPriority tests
    // ============================================================================

    #[test]
    fn task_priority_as_str_returns_correct_strings() {
        assert_eq!(TaskPriority::Low.as_str(), "low");
        assert_eq!(TaskPriority::Medium.as_str(), "medium");
        assert_eq!(TaskPriority::High.as_str(), "high");
        assert_eq!(TaskPriority::Urgent.as_str(), "urgent");
    }

    #[test]
    fn task_priority_parse_valid_values() {
        assert_eq!(TaskPriority::parse("low"), Some(TaskPriority::Low));
        assert_eq!(TaskPriority::parse("medium"), Some(TaskPriority::Medium));
        assert_eq!(TaskPriority::parse("high"), Some(TaskPriority::High));
        assert_eq!(TaskPriority::parse("urgent"), Some(TaskPriority::Urgent));
    }

    #[test]
    fn task_priority_parse_invalid_values() {
        assert_eq!(TaskPriority::parse("unknown"), None);
        assert_eq!(TaskPriority::parse("LOW"), None); // case sensitive
        assert_eq!(TaskPriority::parse("critical"), None); // wrong name
    }

    #[test]
    fn task_priority_roundtrip() {
        for priority in [
            TaskPriority::Low,
            TaskPriority::Medium,
            TaskPriority::High,
            TaskPriority::Urgent,
        ] {
            let s = priority.as_str();
            let parsed = TaskPriority::parse(s).expect("should parse own output");
            assert_eq!(parsed, priority);
        }
    }

    // ============================================================================
    // Serialization tests
    // ============================================================================

    #[test]
    fn task_status_serializes_to_snake_case() {
        let status = TaskStatus::InProgress;
        let json = serde_json::to_string(&status).expect("should serialize");
        assert_eq!(json, "\"in_progress\"");
    }

    #[test]
    fn task_status_deserializes_from_snake_case() {
        let status: TaskStatus =
            serde_json::from_str("\"in_progress\"").expect("should deserialize");
        assert_eq!(status, TaskStatus::InProgress);
    }

    #[test]
    fn project_status_serializes_to_snake_case() {
        let status = ProjectStatus::Active;
        let json = serde_json::to_string(&status).expect("should serialize");
        assert_eq!(json, "\"active\"");
    }

    #[test]
    fn task_priority_serializes_to_snake_case() {
        let priority = TaskPriority::High;
        let json = serde_json::to_string(&priority).expect("should serialize");
        assert_eq!(json, "\"high\"");
    }
}
