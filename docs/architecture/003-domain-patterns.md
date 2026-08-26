# Domain Patterns

This document describes common domain patterns used in the Acme reference implementation.

## Overview

The reference implementation demonstrates these patterns:

- CRUD operations with soft delete
- Manual ordering with weight columns
- Batch operations for bulk actions
- Activity logging for audit trails
- Search and filtering
- Pagination

## Soft Delete Pattern

### Database Schema

```sql
CREATE TABLE acme.projects (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ  -- NULL means not deleted
);

-- Index for efficient filtering
CREATE INDEX idx_projects_deleted_at ON acme.projects(deleted_at);
```

### Query Patterns

```rust
// Regular queries exclude deleted items
pub async fn get_project(pool: &DbPool, id: Uuid) -> Result<Option<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        r#"
        SELECT id, name, status, created_at, updated_at
        FROM acme.projects
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

// Admin queries can include deleted items
pub async fn get_project_admin(pool: &DbPool, id: Uuid) -> Result<Option<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        r#"
        SELECT id, name, status, created_at, updated_at, deleted_at
        FROM acme.projects
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
```

### Soft Delete & Restore

```rust
// Soft delete
pub async fn soft_delete_project(pool: &DbPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE acme.projects
        SET deleted_at = NOW(), updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

// Restore
pub async fn restore_project(pool: &DbPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE acme.projects
        SET deleted_at = NULL, updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NOT NULL
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

// Permanent delete (purge)
pub async fn purge_project(pool: &DbPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"DELETE FROM acme.projects WHERE id = $1"#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
```

## Manual Ordering Pattern

Scope: only canonical, persisted-order entities are reorderable in Acme admin.

- Reorderable:
  - projects
  - categories
  - tasks (within project)
- Intentionally not reorderable:
  - labels (name-oriented list, not canonical manual order)
  - date/computed sorted views (activity, jobs, error logs, dashboard aggregates)

### Database Schema

```sql
CREATE TABLE acme.categories (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    weight INTEGER NOT NULL DEFAULT 0,  -- Lower = higher priority
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for efficient ordering
CREATE INDEX idx_categories_weight ON acme.categories(weight);
```

### Reordering

```rust
#[derive(Deserialize)]
pub struct ReorderItem {
    pub id: Uuid,
    pub weight: i32,
}

pub async fn reorder_categories(
    pool: &DbPool,
    items: Vec<ReorderItem>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    for item in items {
        sqlx::query(
            r#"
            UPDATE acme.categories
            SET weight = $2, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(item.id)
        .bind(item.weight)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
```

### Conflict Recovery Contract

Reorder APIs return `409 Conflict` when list membership changed between load and save, with machine-readable context:

- `added_ids`: IDs that now exist server-side but were missing from submission.
- `removed_ids`: IDs submitted by client that no longer exist server-side.

Admin reorder UI applies this context to the pending reorder state, keeps reorder mode active, and asks the operator to review and save again.

### Frontend Drag & Drop

```svelte
<script lang="ts">
  import { flip } from "svelte/animate";

  let items = $state([...]);
  let draggedItem = $state<Item | null>(null);

  function handleDragStart(item: Item) {
    draggedItem = item;
  }

  function handleDrop(targetIndex: number) {
    if (!draggedItem) return;

    const oldIndex = items.findIndex((i) => i.id === draggedItem!.id);
    items.splice(oldIndex, 1);
    items.splice(targetIndex, 0, draggedItem);

    // Update weights and save
    const reorderPayload = items.map((item, index) => ({
      id: item.id,
      weight: index,
    }));

    saveOrder(reorderPayload);
    draggedItem = null;
  }
</script>

<ul>
  {#each items as item, index (item.id)}
    <li
      animate:flip={{ duration: 200 }}
      draggable="true"
      ondragstart={() => handleDragStart(item)}
      ondrop={() => handleDrop(index)}
    >
      {item.name}
    </li>
  {/each}
</ul>
```

## Batch Operations Pattern

### API Endpoint

```rust
#[derive(Deserialize)]
pub struct BatchDeleteRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Serialize)]
pub struct BatchDeleteResult {
    pub deleted: i64,
    pub failed: Vec<String>,
}

pub async fn batch_delete_projects(
    State(state): State<AppState>,
    AdminUser(user): AdminUser,
    Json(payload): Json<BatchDeleteRequest>,
) -> Result<Json<BatchDeleteResult>, AppError> {
    let mut deleted = 0;
    let mut failed = Vec::new();

    for id in payload.ids {
        match soft_delete_project(state.pool(), id).await {
            Ok(true) => {
                deleted += 1;
                // Log activity
                log_activity(
                    state.pool(),
                    user.id,
                    "delete",
                    "project",
                    id,
                    None,
                ).await?;
            }
            Ok(false) => {
                failed.push(format!("{}: not found", id));
            }
            Err(e) => {
                failed.push(format!("{}: {}", id, e));
            }
        }
    }

    Ok(Json(BatchDeleteResult { deleted, failed }))
}
```

### Frontend Selection

```svelte
<script lang="ts">
  import { Checkbox } from "@inflatable-cookie/poodle-svelte-primitives";

  let items = $state<Item[]>([]);
  let selectedIds = $state<Set<string>>(new Set());

  const allSelected = $derived(
    items.length > 0 && selectedIds.size === items.length
  );

  function toggleAll() {
    if (allSelected) {
      selectedIds = new Set();
    } else {
      selectedIds = new Set(items.map((i) => i.id));
    }
  }

  function toggleItem(id: string) {
    if (selectedIds.has(id)) {
      selectedIds.delete(id);
      selectedIds = new Set(selectedIds);
    } else {
      selectedIds.add(id);
      selectedIds = new Set(selectedIds);
    }
  }

  async function deleteSelected() {
    if (selectedIds.size === 0) return;

    const confirmed = await confirm(
      `Delete ${selectedIds.size} items?`
    );
    if (!confirmed) return;

    await batchDelete([...selectedIds]);
    selectedIds = new Set();
  }
</script>

{#if selectedIds.size > 0}
  <div class="batch-actions">
    <span>{selectedIds.size} selected</span>
    <Button onclick={deleteSelected}>Delete</Button>
  </div>
{/if}

<table>
  <thead>
    <tr>
      <th>
        <Checkbox checked={allSelected} onchange={toggleAll} />
      </th>
      <th>Name</th>
    </tr>
  </thead>
  <tbody>
    {#each items as item (item.id)}
      <tr>
        <td>
          <Checkbox
            checked={selectedIds.has(item.id)}
            onchange={() => toggleItem(item.id)}
          />
        </td>
        <td>{item.name}</td>
      </tr>
    {/each}
  </tbody>
</table>
```

## Activity Logging Pattern

### Database Schema

```sql
CREATE TABLE platform.audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id UUID REFERENCES auth.users(id),
    actor_type TEXT NOT NULL,      -- 'user', 'system', 'job'
    action TEXT NOT NULL,          -- 'create', 'update', 'delete'
    entity_type TEXT NOT NULL,     -- 'project', 'task', 'user'
    entity_id UUID NOT NULL,
    details JSONB,                 -- Additional context
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for common queries
CREATE INDEX idx_audit_log_entity ON platform.audit_log(entity_type, entity_id);
CREATE INDEX idx_audit_log_actor ON platform.audit_log(actor_id);
CREATE INDEX idx_audit_log_occurred_at ON platform.audit_log(occurred_at DESC);
```

### Logging Helper

```rust
pub async fn log_activity(
    pool: &DbPool,
    actor_id: Option<Uuid>,
    actor_type: &str,
    action: &str,
    entity_type: &str,
    entity_id: Uuid,
    details: Option<serde_json::Value>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO platform.audit_log
            (actor_id, actor_type, action, entity_type, entity_id, details)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(actor_id)
    .bind(actor_type)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(details)
    .execute(pool)
    .await?;

    Ok(())
}
```

### Usage in Handlers

```rust
pub async fn create_project(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<Json<ProjectResponse>, AppError> {
    let project = db::create_project(
        state.pool(),
        Uuid::now_v7(),
        user.id,
        &payload.name,
        payload.description.as_deref(),
    )
    .await?;

    // Log the activity
    log_activity(
        state.pool(),
        Some(user.id),
        "user",
        "create",
        "project",
        project.id,
        Some(json!({
            "name": project.name,
        })),
    )
    .await?;

    Ok(Json(ProjectResponse::from(project)))
}
```

## Search and Filtering Pattern

### Query Builder

```rust
use underlay_http::query::{FieldMapping, QueryParams, WhereBuilder};

pub fn project_field_mapping() -> FieldMapping {
    FieldMapping::new()
        .map("name", "p.name")
        .map("status", "p.status")
        .sort_only("createdAt", "p.created_at")
        .filter_only("categoryId", "p.category_id")
        .filter_only("ownerId", "p.owner_id")
}

pub async fn list_projects(
    pool: &DbPool,
    params: &QueryParams,
) -> Result<Vec<ProjectRow>, sqlx::Error> {
    let mapping = project_field_mapping();
    let mut builder = WhereBuilder::new();

    // Apply filters from query params
    builder.apply_filters(params, &mapping);

    // Add default conditions
    builder.add_raw("p.deleted_at IS NULL");

    let sql = format!(
        r#"
        SELECT p.id, p.name, p.status, p.created_at, p.updated_at
        FROM acme.projects p
        {}
        {}
        LIMIT $1 OFFSET $2
        "#,
        builder.to_sql(),
        mapping.order_by_clause(params),
    );

    sqlx::query_as::<_, ProjectRow>(&sql)
        .bind(params.limit())
        .bind(params.offset())
        .fetch_all(pool)
        .await
}
```

### API Endpoint

```rust
#[derive(Deserialize)]
pub struct ListProjectsQuery {
    pub status: Option<String>,
    pub category_id: Option<Uuid>,
    pub q: Option<String>,           // Search query
    pub sort_by: Option<String>,     // Field to sort by
    pub sort_dir: Option<String>,    // 'asc' or 'desc'
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_projects(
    State(state): State<AppState>,
    Query(query): Query<ListProjectsQuery>,
) -> Result<Json<ListResponse<ProjectResponse>>, AppError> {
    let params = QueryParams::from_query(&query);

    let projects = db::list_projects(state.pool(), &params).await?;
    let total = db::count_projects(state.pool(), &params).await?;

    Ok(Json(ListResponse {
        data: projects.into_iter().map(ProjectResponse::from).collect(),
        has_more: params.offset() + projects.len() as i64 < total,
        total,
    }))
}
```

### Frontend URL State

```svelte
<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";

  // Read filters from URL
  const status = $derived($page.url.searchParams.get("status") ?? "");
  const search = $derived($page.url.searchParams.get("q") ?? "");
  const sortBy = $derived($page.url.searchParams.get("sortBy") ?? "createdAt");

  // Update URL when filters change
  function updateFilters(updates: Record<string, string | null>) {
    const url = new URL($page.url);

    for (const [key, value] of Object.entries(updates)) {
      if (value) {
        url.searchParams.set(key, value);
      } else {
        url.searchParams.delete(key);
      }
    }

    goto(url.toString(), { replaceState: true });
  }
</script>

<FilterBar>
  <Select
    value={status}
    onchange={(e) => updateFilters({ status: e.target.value })}
    items={[
      { value: "", label: "All statuses" },
      { value: "active", label: "Active" },
      { value: "archived", label: "Archived" },
    ]}
  />

  <TextInput
    value={search}
    placeholder="Search..."
    debounce={300}
    onchange={(value) => updateFilters({ q: value })}
  />
</FilterBar>
```

## Pagination Pattern

### Database Query

```rust
#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub has_more: bool,
    pub total: i64,
}

pub async fn list_projects_paginated(
    pool: &DbPool,
    limit: i64,
    offset: i64,
) -> Result<PaginatedResponse<ProjectRow>, sqlx::Error> {
    let data = sqlx::query_as::<_, ProjectRow>(
        r#"
        SELECT id, name, status, created_at, updated_at
        FROM acme.projects
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM acme.projects
        WHERE deleted_at IS NULL
        "#,
    )
    .fetch_one(pool)
    .await?;

    Ok(PaginatedResponse {
        has_more: offset + (data.len() as i64) < total,
        data,
        total,
    })
}
```

### Frontend Pagination Component

```svelte
<script lang="ts">
  interface Props {
    page: number;
    pageSize: number;
    total: number;
    onPageChange: (page: number) => void;
  }

  let { page, pageSize, total, onPageChange }: Props = $props();

  const totalPages = $derived(Math.ceil(total / pageSize));
  const canPrev = $derived(page > 1);
  const canNext = $derived(page < totalPages);
</script>

<div class="pagination">
  <Button
    disabled={!canPrev}
    onclick={() => onPageChange(page - 1)}
  >
    Previous
  </Button>

  <span>Page {page} of {totalPages}</span>

  <Button
    disabled={!canNext}
    onclick={() => onPageChange(page + 1)}
  >
    Next
  </Button>
</div>
```

## TypeScript Types

```typescript
// Common response wrapper
export interface ListResponse<T> {
  data: T[];
  hasMore: boolean;
  total: number;
}

// Query parameters
export interface ListParams {
  limit?: number;
  offset?: number;
  sortBy?: string;
  sortDir?: "asc" | "desc";
  q?: string;
  [key: string]: string | number | undefined;
}

// Build query string from params
export function buildQueryString(params: ListParams): string {
  const entries = Object.entries(params)
    .filter(([, v]) => v !== undefined && v !== "")
    .map(([k, v]) => `${k}=${encodeURIComponent(String(v))}`);

  return entries.length > 0 ? `?${entries.join("&")}` : "";
}
```
