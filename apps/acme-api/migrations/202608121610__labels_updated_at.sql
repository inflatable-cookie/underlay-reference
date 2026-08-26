-- Add updated_at to acme.labels so label detail/update routes can emit
-- ETags and honor If-Match like the tasks/categories resources.

ALTER TABLE acme.labels
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
