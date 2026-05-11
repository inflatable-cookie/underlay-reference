
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
    assert_eq!(
        TaskStatus::parse("in_progress"),
        Some(TaskStatus::InProgress)
    );
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
    let status: TaskStatus = serde_json::from_str("\"in_progress\"").expect("should deserialize");
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
