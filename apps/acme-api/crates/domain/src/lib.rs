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
#[path = "tests/lib_tests.rs"]
mod tests;
