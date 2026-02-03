-- Dev seeds: Acme domain schema
--
-- Categories, projects, tasks, labels, and tags for development testing.

-- =========================================
-- acme.categories
-- =========================================

INSERT INTO acme.categories (id, name, slug, description, color, weight, is_active)
VALUES
  ('018f2a3b-0001-7e8f-8a9b-000000000001'::uuid,
   'Development',
   'development',
   'Software development tasks and projects',
   '#3b82f6',
   0,
   TRUE),
  ('018f2a3b-0001-7e8f-8a9b-000000000002'::uuid,
   'Design',
   'design',
   'UI/UX design and graphics work',
   '#8b5cf6',
   1,
   TRUE),
  ('018f2a3b-0001-7e8f-8a9b-000000000003'::uuid,
   'Marketing',
   'marketing',
   'Marketing campaigns and content',
   '#ec4899',
   2,
   TRUE),
  ('018f2a3b-0001-7e8f-8a9b-000000000004'::uuid,
   'Operations',
   'operations',
   'Internal operations and infrastructure',
   '#f59e0b',
   3,
   TRUE),
  ('018f2a3b-0001-7e8f-8a9b-000000000005'::uuid,
   'Archive',
   'archive',
   'Archived/inactive category',
   '#6b7280',
   99,
   FALSE)
ON CONFLICT (slug) DO UPDATE
  SET name = EXCLUDED.name,
      description = EXCLUDED.description,
      color = EXCLUDED.color,
      weight = EXCLUDED.weight,
      is_active = EXCLUDED.is_active,
      updated_at = NOW();

-- =========================================
-- acme.projects
-- =========================================

INSERT INTO acme.projects (id, owner_id, category_id, name, description, status, weight)
VALUES
  -- Development projects
  ('018f2a3b-0002-7e8f-8a9b-000000000001'::uuid,
   '018f2a3b-3c4d-7e8f-8a9b-00000000a001'::uuid,
   '018f2a3b-0001-7e8f-8a9b-000000000001'::uuid,
   'API v2',
   'Next generation API with improved performance',
   'active',
   0),
  ('018f2a3b-0002-7e8f-8a9b-000000000002'::uuid,
   '018f2a3b-3c4d-7e8f-8a9b-00000000a001'::uuid,
   '018f2a3b-0001-7e8f-8a9b-000000000001'::uuid,
   'Mobile App',
   'React Native mobile application',
   'active',
   1),
  -- Design projects
  ('018f2a3b-0002-7e8f-8a9b-000000000003'::uuid,
   '018f2a3b-3c4d-7e8f-8a9b-00000000a002'::uuid,
   '018f2a3b-0001-7e8f-8a9b-000000000002'::uuid,
   'Brand Refresh',
   'Update brand guidelines and assets',
   'active',
   0),
  -- Marketing projects
  ('018f2a3b-0002-7e8f-8a9b-000000000004'::uuid,
   '018f2a3b-3c4d-7e8f-8a9b-00000000a002'::uuid,
   '018f2a3b-0001-7e8f-8a9b-000000000003'::uuid,
   'Q1 Campaign',
   'First quarter marketing campaign',
   'active',
   0),
  -- Archived project
  ('018f2a3b-0002-7e8f-8a9b-000000000005'::uuid,
   '018f2a3b-3c4d-7e8f-8a9b-00000000a001'::uuid,
   '018f2a3b-0001-7e8f-8a9b-000000000001'::uuid,
   'Legacy Migration',
   'Completed legacy system migration',
   'archived',
   99)
ON CONFLICT (id) DO UPDATE
  SET name = EXCLUDED.name,
      description = EXCLUDED.description,
      category_id = EXCLUDED.category_id,
      status = EXCLUDED.status,
      weight = EXCLUDED.weight,
      updated_at = NOW();

-- =========================================
-- acme.labels (project-specific)
-- =========================================

INSERT INTO acme.labels (id, project_id, name, color, weight)
VALUES
  -- Labels for API v2 project
  ('018f2a3b-0003-7e8f-8a9b-000000000001'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000001'::uuid,
   'Bug',
   '#ef4444',
   0),
  ('018f2a3b-0003-7e8f-8a9b-000000000002'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000001'::uuid,
   'Feature',
   '#22c55e',
   1),
  ('018f2a3b-0003-7e8f-8a9b-000000000003'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000001'::uuid,
   'Enhancement',
   '#3b82f6',
   2),
  ('018f2a3b-0003-7e8f-8a9b-000000000004'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000001'::uuid,
   'Documentation',
   '#a855f7',
   3),
  -- Labels for Mobile App project
  ('018f2a3b-0003-7e8f-8a9b-000000000005'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000002'::uuid,
   'iOS',
   '#6366f1',
   0),
  ('018f2a3b-0003-7e8f-8a9b-000000000006'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000002'::uuid,
   'Android',
   '#22c55e',
   1),
  ('018f2a3b-0003-7e8f-8a9b-000000000007'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000002'::uuid,
   'Critical',
   '#ef4444',
   2)
ON CONFLICT ON CONSTRAINT labels_unique_name_per_project DO UPDATE
  SET color = EXCLUDED.color,
      weight = EXCLUDED.weight;

-- =========================================
-- acme.tags (project-level organization)
-- =========================================

INSERT INTO acme.tags (id, project_id, name, color, weight)
VALUES
  ('018f2a3b-0004-7e8f-8a9b-000000000001'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000001'::uuid,
   'backend',
   '#f59e0b',
   0),
  ('018f2a3b-0004-7e8f-8a9b-000000000002'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000001'::uuid,
   'frontend',
   '#06b6d4',
   1),
  ('018f2a3b-0004-7e8f-8a9b-000000000003'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000001'::uuid,
   'database',
   '#84cc16',
   2)
ON CONFLICT ON CONSTRAINT tags_unique_name_per_project DO UPDATE
  SET color = EXCLUDED.color,
      weight = EXCLUDED.weight;

-- =========================================
-- acme.tasks
-- =========================================

INSERT INTO acme.tasks (id, project_id, title, description, status, priority, due_date, position, weight)
VALUES
  -- API v2 tasks
  ('018f2a3b-0005-7e8f-8a9b-000000000001'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000001'::uuid,
   'Design new authentication flow',
   'Create OAuth2 + PKCE authentication with refresh token rotation',
   'completed',
   'high',
   '2026-01-15',
   0,
   0),
  ('018f2a3b-0005-7e8f-8a9b-000000000002'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000001'::uuid,
   'Implement rate limiting',
   'Add configurable rate limiting per endpoint with Redis backend',
   'in_progress',
   'high',
   '2026-02-01',
   1,
   1),
  ('018f2a3b-0005-7e8f-8a9b-000000000003'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000001'::uuid,
   'Add GraphQL support',
   'Evaluate and implement GraphQL alongside REST endpoints',
   'pending',
   'medium',
   '2026-03-01',
   2,
   2),
  ('018f2a3b-0005-7e8f-8a9b-000000000004'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000001'::uuid,
   'Write API documentation',
   'OpenAPI spec and developer guide',
   'pending',
   'low',
   NULL,
   3,
   3),
  -- Mobile App tasks
  ('018f2a3b-0005-7e8f-8a9b-000000000005'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000002'::uuid,
   'Setup React Native project',
   'Initialize project with TypeScript and navigation',
   'completed',
   'high',
   '2026-01-10',
   0,
   0),
  ('018f2a3b-0005-7e8f-8a9b-000000000006'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000002'::uuid,
   'Implement login screen',
   'Build login UI with biometric authentication option',
   'in_progress',
   'high',
   '2026-01-25',
   1,
   1),
  ('018f2a3b-0005-7e8f-8a9b-000000000007'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000002'::uuid,
   'Build dashboard view',
   'Main dashboard with key metrics and quick actions',
   'pending',
   'medium',
   '2026-02-15',
   2,
   2),
  -- Brand Refresh tasks
  ('018f2a3b-0005-7e8f-8a9b-000000000008'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000003'::uuid,
   'Create color palette',
   'Define primary, secondary, and accent colors',
   'completed',
   'high',
   '2026-01-05',
   0,
   0),
  ('018f2a3b-0005-7e8f-8a9b-000000000009'::uuid,
   '018f2a3b-0002-7e8f-8a9b-000000000003'::uuid,
   'Design new logo',
   'Modern logo that works across all media',
   'in_progress',
   'urgent',
   '2026-01-20',
   1,
   1)
ON CONFLICT (id) DO UPDATE
  SET title = EXCLUDED.title,
      description = EXCLUDED.description,
      status = EXCLUDED.status,
      priority = EXCLUDED.priority,
      due_date = EXCLUDED.due_date,
      position = EXCLUDED.position,
      weight = EXCLUDED.weight,
      updated_at = NOW();

-- =========================================
-- acme.task_labels (assign labels to tasks)
-- =========================================

INSERT INTO acme.task_labels (task_id, label_id)
VALUES
  -- API v2 tasks
  ('018f2a3b-0005-7e8f-8a9b-000000000001'::uuid, '018f2a3b-0003-7e8f-8a9b-000000000002'::uuid), -- Feature
  ('018f2a3b-0005-7e8f-8a9b-000000000002'::uuid, '018f2a3b-0003-7e8f-8a9b-000000000002'::uuid), -- Feature
  ('018f2a3b-0005-7e8f-8a9b-000000000003'::uuid, '018f2a3b-0003-7e8f-8a9b-000000000002'::uuid), -- Feature
  ('018f2a3b-0005-7e8f-8a9b-000000000004'::uuid, '018f2a3b-0003-7e8f-8a9b-000000000004'::uuid), -- Documentation
  -- Mobile App tasks
  ('018f2a3b-0005-7e8f-8a9b-000000000005'::uuid, '018f2a3b-0003-7e8f-8a9b-000000000005'::uuid), -- iOS
  ('018f2a3b-0005-7e8f-8a9b-000000000005'::uuid, '018f2a3b-0003-7e8f-8a9b-000000000006'::uuid), -- Android
  ('018f2a3b-0005-7e8f-8a9b-000000000006'::uuid, '018f2a3b-0003-7e8f-8a9b-000000000005'::uuid), -- iOS
  ('018f2a3b-0005-7e8f-8a9b-000000000006'::uuid, '018f2a3b-0003-7e8f-8a9b-000000000006'::uuid)  -- Android
ON CONFLICT (task_id, label_id) DO NOTHING;

-- =========================================
-- acme.task_tags (assign tags to tasks)
-- =========================================

INSERT INTO acme.task_tags (task_id, tag_id)
VALUES
  ('018f2a3b-0005-7e8f-8a9b-000000000001'::uuid, '018f2a3b-0004-7e8f-8a9b-000000000001'::uuid), -- backend
  ('018f2a3b-0005-7e8f-8a9b-000000000002'::uuid, '018f2a3b-0004-7e8f-8a9b-000000000001'::uuid), -- backend
  ('018f2a3b-0005-7e8f-8a9b-000000000003'::uuid, '018f2a3b-0004-7e8f-8a9b-000000000001'::uuid), -- backend
  ('018f2a3b-0005-7e8f-8a9b-000000000003'::uuid, '018f2a3b-0004-7e8f-8a9b-000000000002'::uuid)  -- frontend
ON CONFLICT (task_id, tag_id) DO NOTHING;

-- =========================================
-- acme.task_assignees (assign users to tasks)
-- =========================================

INSERT INTO acme.task_assignees (task_id, user_id)
VALUES
  ('018f2a3b-0005-7e8f-8a9b-000000000001'::uuid, '018f2a3b-3c4d-7e8f-8a9b-00000000a001'::uuid), -- Admin
  ('018f2a3b-0005-7e8f-8a9b-000000000002'::uuid, '018f2a3b-3c4d-7e8f-8a9b-00000000a001'::uuid), -- Admin
  ('018f2a3b-0005-7e8f-8a9b-000000000002'::uuid, '018f2a3b-3c4d-7e8f-8a9b-00000000a003'::uuid), -- Alice
  ('018f2a3b-0005-7e8f-8a9b-000000000003'::uuid, '018f2a3b-3c4d-7e8f-8a9b-00000000a003'::uuid), -- Alice
  ('018f2a3b-0005-7e8f-8a9b-000000000005'::uuid, '018f2a3b-3c4d-7e8f-8a9b-00000000a004'::uuid), -- Bob
  ('018f2a3b-0005-7e8f-8a9b-000000000006'::uuid, '018f2a3b-3c4d-7e8f-8a9b-00000000a004'::uuid), -- Bob
  ('018f2a3b-0005-7e8f-8a9b-000000000008'::uuid, '018f2a3b-3c4d-7e8f-8a9b-00000000a002'::uuid), -- User
  ('018f2a3b-0005-7e8f-8a9b-000000000009'::uuid, '018f2a3b-3c4d-7e8f-8a9b-00000000a002'::uuid)  -- User
ON CONFLICT (task_id, user_id) DO NOTHING;

-- =========================================
-- acme.task_comments
-- =========================================

INSERT INTO acme.task_comments (id, task_id, author_id, body)
VALUES
  ('018f2a3b-0006-7e8f-8a9b-000000000001'::uuid,
   '018f2a3b-0005-7e8f-8a9b-000000000002'::uuid,
   '018f2a3b-3c4d-7e8f-8a9b-00000000a001'::uuid,
   'Started implementing the Redis backend for rate limiting. Using sliding window algorithm.'),
  ('018f2a3b-0006-7e8f-8a9b-000000000002'::uuid,
   '018f2a3b-0005-7e8f-8a9b-000000000002'::uuid,
   '018f2a3b-3c4d-7e8f-8a9b-00000000a003'::uuid,
   'Should we also add per-user rate limits in addition to per-endpoint?'),
  ('018f2a3b-0006-7e8f-8a9b-000000000003'::uuid,
   '018f2a3b-0005-7e8f-8a9b-000000000009'::uuid,
   '018f2a3b-3c4d-7e8f-8a9b-00000000a002'::uuid,
   'First draft of the logo is ready for review. Check the shared Figma file.')
ON CONFLICT (id) DO UPDATE
  SET body = EXCLUDED.body,
      updated_at = NOW();
