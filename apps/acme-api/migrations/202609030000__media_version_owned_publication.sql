-- Private owned-publication identity for Underlay v0.9.7 recovery.
-- Token and destination key stay off public DTOs. Provider/bucket already exist.

ALTER TABLE media.media_version
    ADD COLUMN IF NOT EXISTS ownership_token bytea NULL,
    ADD COLUMN IF NOT EXISTS published_object_key text NULL
        CHECK (published_object_key IS NULL OR char_length(published_object_key) <= 512);

ALTER TABLE media.media_version
    DROP CONSTRAINT IF EXISTS media_version_owned_authority_complete;

ALTER TABLE media.media_version
    ADD CONSTRAINT media_version_owned_authority_complete
    CHECK (
        (ownership_token IS NULL AND published_object_key IS NULL)
        OR (
            ownership_token IS NOT NULL
            AND octet_length(ownership_token) >= 32
            AND published_object_key IS NOT NULL
            AND storage_provider IS NOT NULL
            AND bucket IS NOT NULL
        )
    );
