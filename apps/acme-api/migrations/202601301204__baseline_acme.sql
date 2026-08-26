-- Acme baseline: domain tables.
--
-- Contains all acme domain tables:
-- - categories (hierarchical grouping for projects)
-- - projects (task containers)
-- - tasks (work items)
-- - task_comments (discussion on tasks)
-- - tags (project-level task organization)
-- - task_tags (junction table)
-- - labels (project-level labeling)
-- - task_labels (junction table)
-- - task_assignees (user assignment)

-- =========================================
-- Categories (parent of projects)
-- =========================================

CREATE TABLE IF NOT EXISTS acme.categories (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    color TEXT,
    weight INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    delete_batch_id UUID
);

CREATE INDEX IF NOT EXISTS idx_acme_categories_active
    ON acme.categories (is_active, weight)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_acme_categories_slug
    ON acme.categories (slug)
    WHERE deleted_at IS NULL;

-- =========================================
-- Projects (task containers)
-- =========================================

CREATE TABLE IF NOT EXISTS acme.projects (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    category_id UUID REFERENCES acme.categories(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    description JSONB,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'archived')),
    weight INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    delete_batch_id UUID
);

CREATE INDEX IF NOT EXISTS idx_acme_projects_owner
    ON acme.projects (owner_id, status)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_acme_projects_category
    ON acme.projects (category_id, weight)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_acme_projects_status_weight
    ON acme.projects (status, weight)
    WHERE deleted_at IS NULL;

-- =========================================
-- Tasks
-- =========================================

CREATE TABLE IF NOT EXISTS acme.tasks (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES acme.projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    notes JSONB,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'completed', 'cancelled')),
    priority TEXT NOT NULL DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
    due_date DATE,
    completed_at TIMESTAMPTZ,
    position INTEGER NOT NULL DEFAULT 0,
    weight INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    delete_batch_id UUID
);

CREATE INDEX IF NOT EXISTS idx_acme_tasks_project
    ON acme.tasks (project_id, status, position)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_acme_tasks_due_date
    ON acme.tasks (due_date)
    WHERE status NOT IN ('completed', 'cancelled') AND deleted_at IS NULL;

-- =========================================
-- Task comments
-- =========================================

CREATE TABLE IF NOT EXISTS acme.task_comments (
    id UUID PRIMARY KEY,
    task_id UUID NOT NULL REFERENCES acme.tasks(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_acme_task_comments_task
    ON acme.task_comments (task_id, created_at);

-- =========================================
-- Tags (for organizing tasks at project level)
-- =========================================

CREATE TABLE IF NOT EXISTS acme.tags (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES acme.projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    color TEXT,
    weight INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    delete_batch_id UUID,
    CONSTRAINT tags_unique_name_per_project UNIQUE (project_id, name)
);

CREATE INDEX IF NOT EXISTS idx_acme_tags_project
    ON acme.tags (project_id, weight)
    WHERE deleted_at IS NULL;

-- Task-tag junction table
CREATE TABLE IF NOT EXISTS acme.task_tags (
    task_id UUID NOT NULL REFERENCES acme.tasks(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES acme.tags(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, tag_id)
);

-- =========================================
-- Labels (multi-select relation for tasks)
-- =========================================

CREATE TABLE IF NOT EXISTS acme.labels (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES acme.projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    color TEXT NOT NULL DEFAULT '#6366f1',
    weight INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    delete_batch_id UUID,
    CONSTRAINT labels_unique_name_per_project UNIQUE (project_id, name)
);

CREATE INDEX IF NOT EXISTS idx_acme_labels_project
    ON acme.labels (project_id, weight)
    WHERE deleted_at IS NULL;

-- Task-label junction table
CREATE TABLE IF NOT EXISTS acme.task_labels (
    task_id UUID NOT NULL REFERENCES acme.tasks(id) ON DELETE CASCADE,
    label_id UUID NOT NULL REFERENCES acme.labels(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (task_id, label_id)
);

-- =========================================
-- Task assignees (user assignment)
-- =========================================

CREATE TABLE IF NOT EXISTS acme.task_assignees (
    task_id UUID NOT NULL REFERENCES acme.tasks(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (task_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_acme_task_assignees_user
    ON acme.task_assignees (user_id);
