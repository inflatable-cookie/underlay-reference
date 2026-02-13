-- Media Library schema.
--
-- Standalone media management with versioned blob storage, usage tracking, and renditions.
-- Other entities reference media by stable `media_id`, never by URLs.
--
-- Tables:
--   media.media           - Stable library items (images, PDFs)
--   media.media_version   - Immutable blob versions with storage mapping
--   media.media_rendition - Derived blobs (thumbnails, resizes)
--   media.media_usage     - Tracks where media is referenced

-- =========================================
-- media.media
-- =========================================
-- The stable entity referenced by other tables. Each media item has a stable ID;
-- the actual bytes are in media_version.

CREATE TABLE IF NOT EXISTS media.media (
    id uuid PRIMARY KEY,

    -- Type and visibility
    kind text NOT NULL CHECK (kind IN ('image', 'pdf') AND char_length(kind) <= 16),
    visibility text NOT NULL DEFAULT 'restricted'
        CHECK (visibility IN ('public', 'restricted') AND char_length(visibility) <= 16),

    -- Metadata
    title text NOT NULL CHECK (char_length(title) <= 256),
    original_filename text NULL CHECK (original_filename IS NULL OR char_length(original_filename) <= 256),

    -- Current active version (nullable during initial creation)
    current_version_id uuid NULL,

    -- Audit fields
    created_at timestamptz NOT NULL DEFAULT now(),
    created_by uuid NULL REFERENCES auth.users(id) ON DELETE SET NULL,
    updated_at timestamptz NOT NULL DEFAULT now(),
    updated_by uuid NULL REFERENCES auth.users(id) ON DELETE SET NULL,

    -- Soft delete
    deleted_at timestamptz NULL,
    deleted_by uuid NULL REFERENCES auth.users(id) ON DELETE SET NULL
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS media_kind_idx
    ON media.media (kind)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS media_visibility_idx
    ON media.media (visibility)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS media_created_at_idx
    ON media.media (created_at);

CREATE INDEX IF NOT EXISTS media_updated_at_idx
    ON media.media (updated_at);

CREATE INDEX IF NOT EXISTS media_deleted_at_idx
    ON media.media (deleted_at)
    WHERE deleted_at IS NOT NULL;

-- =========================================
-- media.media_version
-- =========================================
-- Immutable record of uploaded bytes. Each upload (including replacements)
-- creates a new version. The media.current_version_id points to the active one.

CREATE TABLE IF NOT EXISTS media.media_version (
    id uuid PRIMARY KEY,
    media_id uuid NOT NULL REFERENCES media.media(id) ON DELETE CASCADE,

    -- Upload state machine
    state text NOT NULL DEFAULT 'uploading'
        CHECK (state IN ('uploading', 'ready', 'failed', 'purging') AND char_length(state) <= 16),

    -- File metadata (populated on finalisation)
    byte_size bigint NULL CHECK (byte_size IS NULL OR byte_size >= 0),
    mime_type text NULL CHECK (mime_type IS NULL OR char_length(mime_type) <= 128),
    sha256 text NULL CHECK (sha256 IS NULL OR char_length(sha256) = 64),

    -- Storage location (populated on finalisation)
    storage_provider text NULL CHECK (storage_provider IS NULL OR char_length(storage_provider) <= 32),
    bucket text NULL CHECK (bucket IS NULL OR char_length(bucket) <= 128),
    object_key text NULL CHECK (object_key IS NULL OR char_length(object_key) <= 512),

    -- Audit
    created_at timestamptz NOT NULL DEFAULT now(),
    created_by uuid NULL REFERENCES auth.users(id) ON DELETE SET NULL
);

-- Index for looking up versions by media
CREATE INDEX IF NOT EXISTS media_version_media_id_idx
    ON media.media_version (media_id);

-- Index for state-based queries (e.g., find stale uploads)
CREATE INDEX IF NOT EXISTS media_version_state_idx
    ON media.media_version (state);

-- Index for deduplication lookup by hash
CREATE INDEX IF NOT EXISTS media_version_sha256_idx
    ON media.media_version (sha256)
    WHERE sha256 IS NOT NULL AND state = 'ready';

-- Ensure storage location is unique when set
CREATE UNIQUE INDEX IF NOT EXISTS media_version_storage_unique
    ON media.media_version (storage_provider, bucket, object_key)
    WHERE object_key IS NOT NULL;

-- Add foreign key from media to current_version after media_version exists
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint c
        JOIN pg_class t ON t.oid = c.conrelid
        JOIN pg_namespace n ON n.oid = t.relnamespace
        WHERE c.conname = 'media_current_version_fk'
          AND n.nspname = 'media'
          AND t.relname = 'media'
    ) THEN
        ALTER TABLE media.media
            ADD CONSTRAINT media_current_version_fk
            FOREIGN KEY (current_version_id)
            REFERENCES media.media_version(id)
            ON DELETE SET NULL;
    END IF;
END $$;

-- =========================================
-- media.media_rendition
-- =========================================
-- Derived blobs (thumbnails, resized images) for a specific version.
-- Each rendition is immutable; regenerating creates new records.

CREATE TABLE IF NOT EXISTS media.media_rendition (
    id uuid PRIMARY KEY,
    media_version_id uuid NOT NULL REFERENCES media.media_version(id) ON DELETE CASCADE,

    -- Rendition type (e.g., 'thumbnail', 'small', 'medium')
    kind text NOT NULL CHECK (char_length(kind) <= 32),

    -- File metadata
    byte_size bigint NOT NULL CHECK (byte_size >= 0),
    mime_type text NOT NULL CHECK (char_length(mime_type) <= 128),
    width int NULL CHECK (width IS NULL OR width > 0),
    height int NULL CHECK (height IS NULL OR height > 0),

    -- Storage location
    storage_provider text NOT NULL CHECK (char_length(storage_provider) <= 32),
    bucket text NOT NULL CHECK (char_length(bucket) <= 128),
    object_key text NOT NULL CHECK (char_length(object_key) <= 512),

    -- Audit
    created_at timestamptz NOT NULL DEFAULT now()
);

-- Index for looking up renditions by version
CREATE INDEX IF NOT EXISTS media_rendition_version_id_idx
    ON media.media_rendition (media_version_id);

-- Ensure only one rendition of each kind per version
CREATE UNIQUE INDEX IF NOT EXISTS media_rendition_unique_kind
    ON media.media_rendition (media_version_id, kind);

-- =========================================
-- media.media_usage
-- =========================================
-- Tracks where media is referenced. Used for:
-- - "Where is this used?" queries
-- - Preventing deletion while in use
-- - Finding orphaned/unused media

CREATE TABLE IF NOT EXISTS media.media_usage (
    id uuid PRIMARY KEY,
    media_id uuid NOT NULL REFERENCES media.media(id) ON DELETE CASCADE,

    -- What entity references this media
    used_by_type text NOT NULL CHECK (char_length(used_by_type) <= 64),
    used_by_id uuid NOT NULL,

    -- Which field/slot in the entity (e.g., 'cover_image', 'attachment', 'body_block')
    field text NOT NULL CHECK (char_length(field) <= 64),

    -- Audit
    created_at timestamptz NOT NULL DEFAULT now()
);

-- Index for looking up usages by media
CREATE INDEX IF NOT EXISTS media_usage_media_id_idx
    ON media.media_usage (media_id);

-- Index for looking up what media an entity uses
CREATE INDEX IF NOT EXISTS media_usage_used_by_idx
    ON media.media_usage (used_by_type, used_by_id);

-- Prevent duplicate usage records
CREATE UNIQUE INDEX IF NOT EXISTS media_usage_unique
    ON media.media_usage (media_id, used_by_type, used_by_id, field);
