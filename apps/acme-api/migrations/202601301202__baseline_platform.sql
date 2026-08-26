-- Acme baseline: platform schema.
--
-- Contains all platform tables from Underlay:
-- - error_log (HTTP error logging)
-- - job, scheduled_task, job_history (background jobs)
-- - domain_events (event outbox)
-- - audit_log (audit trail)

-- =========================================
-- Error logging (underlay-http)
-- =========================================

CREATE TABLE IF NOT EXISTS platform.error_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- HTTP request context
    endpoint        TEXT        NOT NULL,
    method          TEXT        NOT NULL,
    status_code     INTEGER     NOT NULL,

    -- Error details
    error_code      TEXT        NOT NULL,
    message         TEXT        NOT NULL,

    -- Request tracing
    correlation_id  TEXT        NOT NULL,

    -- Additional context (headers, body, etc.)
    context         JSONB       NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_platform_error_log_occurred_at
    ON platform.error_log (occurred_at DESC);

CREATE INDEX IF NOT EXISTS idx_platform_error_log_status_code
    ON platform.error_log (status_code);

CREATE INDEX IF NOT EXISTS idx_platform_error_log_error_code
    ON platform.error_log (error_code);

CREATE INDEX IF NOT EXISTS idx_platform_error_log_correlation_id
    ON platform.error_log (correlation_id);

CREATE INDEX IF NOT EXISTS idx_platform_error_log_endpoint
    ON platform.error_log (endpoint);

-- =========================================
-- Background jobs (underlay-jobs)
-- =========================================

CREATE TABLE IF NOT EXISTS platform.job (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'claimed', 'running', 'succeeded', 'failed', 'cancelled')),
    payload JSONB NOT NULL DEFAULT '{}',
    attempts INT NOT NULL DEFAULT 0,
    max_attempts INT NOT NULL DEFAULT 1,
    scheduled_for TIMESTAMPTZ,
    priority INT NOT NULL DEFAULT 0,
    claimed_at TIMESTAMPTZ,
    claimed_by TEXT,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    heartbeat_at TIMESTAMPTZ,
    progress JSONB,
    last_error TEXT,
    error_history JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_platform_job_claimable
    ON platform.job (scheduled_for NULLS FIRST, priority DESC, created_at)
    WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_platform_job_running
    ON platform.job (heartbeat_at)
    WHERE status IN ('claimed', 'running');

CREATE INDEX IF NOT EXISTS idx_platform_job_type ON platform.job (job_type, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_platform_job_created ON platform.job (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_platform_job_status ON platform.job (status, created_at DESC);

-- Job updated_at trigger
CREATE OR REPLACE FUNCTION platform.job_update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_platform_job_updated_at
    BEFORE UPDATE ON platform.job
    FOR EACH ROW
    EXECUTE FUNCTION platform.job_update_timestamp();

-- Scheduled tasks
CREATE TABLE IF NOT EXISTS platform.scheduled_task (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    job_type TEXT NOT NULL,
    schedule TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    max_attempts INT NOT NULL DEFAULT 1,
    timeout_seconds INT,
    allow_overlap BOOLEAN NOT NULL DEFAULT FALSE,
    priority INT NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    last_scheduled_at TIMESTAMPTZ,
    last_completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_platform_scheduled_task_enabled
    ON platform.scheduled_task (enabled, name)
    WHERE enabled = TRUE;

CREATE TRIGGER trigger_platform_scheduled_task_updated_at
    BEFORE UPDATE ON platform.scheduled_task
    FOR EACH ROW
    EXECUTE FUNCTION platform.job_update_timestamp();

-- Job history
CREATE TABLE IF NOT EXISTS platform.job_history (
    id UUID PRIMARY KEY,
    job_type TEXT NOT NULL,
    status TEXT NOT NULL,
    payload JSONB,
    attempts INT NOT NULL,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    duration_ms BIGINT,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_platform_job_history_type
    ON platform.job_history (job_type, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_platform_job_history_created
    ON platform.job_history (created_at DESC);

-- Job archive function
CREATE OR REPLACE FUNCTION platform.archive_completed_jobs(older_than_interval INTERVAL)
RETURNS BIGINT AS $$
DECLARE
    archived_count BIGINT;
BEGIN
    INSERT INTO platform.job_history (id, job_type, status, payload, attempts, started_at, finished_at, duration_ms, last_error, created_at)
    SELECT
        id,
        job_type,
        status,
        payload,
        attempts,
        started_at,
        finished_at,
        EXTRACT(EPOCH FROM (finished_at - started_at)) * 1000,
        last_error,
        created_at
    FROM platform.job
    WHERE status IN ('succeeded', 'failed', 'cancelled')
      AND finished_at < NOW() - older_than_interval;

    GET DIAGNOSTICS archived_count = ROW_COUNT;

    DELETE FROM platform.job
    WHERE status IN ('succeeded', 'failed', 'cancelled')
      AND finished_at < NOW() - older_than_interval;

    RETURN archived_count;
END;
$$ LANGUAGE plpgsql;

-- Job history purge function
CREATE OR REPLACE FUNCTION platform.purge_job_history(older_than_interval INTERVAL)
RETURNS BIGINT AS $$
DECLARE
    purged_count BIGINT;
BEGIN
    DELETE FROM platform.job_history
    WHERE created_at < NOW() - older_than_interval;

    GET DIAGNOSTICS purged_count = ROW_COUNT;
    RETURN purged_count;
END;
$$ LANGUAGE plpgsql;

-- Job notify trigger (LISTEN/NOTIFY for efficient wake-up)
CREATE OR REPLACE FUNCTION platform.notify_job_inserted()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify('underlay_job_notify', NEW.job_type);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_platform_job_notify
    AFTER INSERT ON platform.job
    FOR EACH ROW
    WHEN (NEW.status = 'pending')
    EXECUTE FUNCTION platform.notify_job_inserted();

COMMENT ON TABLE platform.job IS 'Background job queue for async task processing';
COMMENT ON TABLE platform.scheduled_task IS 'Definitions of recurring scheduled tasks';
COMMENT ON TABLE platform.job_history IS 'Archived completed jobs for visibility and debugging';
COMMENT ON COLUMN platform.job.status IS 'Job lifecycle: pending -> claimed -> running -> succeeded|failed|cancelled';
COMMENT ON COLUMN platform.job.scheduled_for IS 'When to run the job. NULL means run immediately when claimed.';
COMMENT ON COLUMN platform.job.priority IS 'Higher priority jobs are claimed first. Default 0.';
COMMENT ON COLUMN platform.job.heartbeat_at IS 'Last heartbeat from worker. Used to detect stalled jobs.';
COMMENT ON COLUMN platform.job.progress IS 'Progress for long-running jobs: {current, total, message}';
COMMENT ON COLUMN platform.scheduled_task.schedule IS 'Cron expression: minute hour day month weekday';
COMMENT ON COLUMN platform.scheduled_task.allow_overlap IS 'If false, skip scheduling if previous instance is still running';
COMMENT ON FUNCTION platform.notify_job_inserted() IS 'Sends pg_notify when a new job is inserted, waking up waiting workers';

-- =========================================
-- Domain events (underlay-events)
-- =========================================

CREATE TABLE IF NOT EXISTS platform.domain_events (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  event_type text NOT NULL,
  payload jsonb NOT NULL,
  occurred_at timestamptz NOT NULL DEFAULT NOW(),
  processed_at timestamptz NULL
);

CREATE INDEX IF NOT EXISTS domain_events_unprocessed_idx
  ON platform.domain_events (occurred_at)
  WHERE processed_at IS NULL;

-- LISTEN/NOTIFY trigger for efficient outbox processor wake-up
CREATE OR REPLACE FUNCTION platform.notify_domain_event_inserted()
RETURNS trigger AS $$
BEGIN
    PERFORM pg_notify('underlay_domain_event_notify', NEW.event_type);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS domain_event_inserted ON platform.domain_events;
CREATE TRIGGER domain_event_inserted
    AFTER INSERT ON platform.domain_events
    FOR EACH ROW
    EXECUTE FUNCTION platform.notify_domain_event_inserted();

-- =========================================
-- Audit logging
-- =========================================

CREATE TABLE IF NOT EXISTS platform.audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id UUID NULL REFERENCES auth.users(id),
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id UUID NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    correlation_id TEXT,
    ip_address TEXT
);

CREATE INDEX IF NOT EXISTS idx_platform_audit_log_occurred_at
    ON platform.audit_log (occurred_at DESC);

CREATE INDEX IF NOT EXISTS idx_platform_audit_log_user_id
    ON platform.audit_log (user_id)
    WHERE user_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_platform_audit_log_resource
    ON platform.audit_log (resource_type, resource_id);

CREATE INDEX IF NOT EXISTS idx_platform_audit_log_action
    ON platform.audit_log (action);
