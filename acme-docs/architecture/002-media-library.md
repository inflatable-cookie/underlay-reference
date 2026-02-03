# Media Library Architecture

This document describes the media library patterns implemented in the Acme reference.

## Overview

The media library provides:

- Versioned file uploads with blob storage
- Client-side deduplication via SHA-256 hashing
- Server-side MIME type validation (magic bytes)
- Automatic thumbnail generation for images
- Soft delete with configurable retention
- Usage tracking for referential integrity

## Upload Flow

```
┌────────────────────────────────────────────────────────────────────────────┐
│                           Upload Flow                                       │
│                                                                             │
│  ┌─────────┐      ┌─────────┐      ┌─────────┐      ┌─────────┐           │
│  │ Client  │      │   API   │      │Database │      │  Blob   │           │
│  └────┬────┘      └────┬────┘      └────┬────┘      └────┬────┘           │
│       │                │                │                │                 │
│       │ 1. Hash file   │                │                │                 │
│       │ (SHA-256)      │                │                │                 │
│       │                │                │                │                 │
│       │ 2. Check dup   │                │                │                 │
│       │───────────────>│                │                │                 │
│       │                │ Query by hash  │                │                 │
│       │                │───────────────>│                │                 │
│       │                │<───────────────│                │                 │
│       │<───────────────│                │                │                 │
│       │                │                │                │                 │
│       │ 3. Create      │                │                │                 │
│       │    media       │                │                │                 │
│       │───────────────>│                │                │                 │
│       │                │ Insert record  │                │                 │
│       │                │───────────────>│                │                 │
│       │<───────────────│                │                │                 │
│       │                │                │                │                 │
│       │ 4. Initiate    │                │                │                 │
│       │    upload      │                │                │                 │
│       │───────────────>│                │                │                 │
│       │                │ Create version │                │                 │
│       │                │───────────────>│                │                 │
│       │                │                │ Get upload URL │                 │
│       │                │                │───────────────>│                 │
│       │<───────────────│                │                │                 │
│       │ {uploadUrl,    │                │                │                 │
│       │  versionId}    │                │                │                 │
│       │                │                │                │                 │
│       │ 5. Upload to   │                │                │                 │
│       │    blob        │                │                │                 │
│       │────────────────────────────────────────────────>│                 │
│       │<────────────────────────────────────────────────│                 │
│       │                │                │                │                 │
│       │ 6. Finalise    │                │                │                 │
│       │───────────────>│                │                │                 │
│       │                │ Verify hash    │                │                 │
│       │                │ Verify size    │                │                 │
│       │                │ Verify MIME    │                │                 │
│       │                │───────────────>│                │                 │
│       │                │                │                │                 │
│       │                │ Queue thumbnail│                │                 │
│       │                │───────────────>│ (jobs table)   │                 │
│       │                │                │                │                 │
│       │<───────────────│                │                │                 │
│       │ {media}        │                │                │                 │
│       │                │                │                │                 │
└────────────────────────────────────────────────────────────────────────────┘
```

## Deduplication

### Client-Side Hashing

Files are hashed before upload to detect duplicates:

```typescript
import { computeFileHash, mediaCommands } from "acme-client";

// Compute SHA-256 hash
const hash = await computeFileHash(file);

// Check for existing file with same hash
const duplicate = await mediaCommands.checkDuplicate(fetch, token, { sha256: hash });

if (duplicate.exists) {
  // File already uploaded - use existing media
  return duplicate.existingMedia;
}

// Proceed with upload
```

### Hash Storage

```sql
-- media_versions table stores hash for each version
CREATE TABLE media.media_versions (
    id UUID PRIMARY KEY,
    media_id UUID NOT NULL REFERENCES media.media(id),
    version_number INTEGER NOT NULL,
    sha256_hash TEXT,  -- SHA-256 hash for deduplication
    file_size BIGINT NOT NULL,
    mime_type TEXT NOT NULL,
    blob_key TEXT,
    state TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for fast duplicate lookups
CREATE INDEX idx_media_versions_sha256 ON media.media_versions(sha256_hash);
```

## File Validation

### Client-Side Validation

```typescript
const ALLOWED_TYPES = [
  "image/jpeg",
  "image/png",
  "image/gif",
  "image/webp",
  "image/svg+xml",
  "application/pdf",
];

const MAX_FILE_SIZE = 50 * 1024 * 1024; // 50MB

function validateFile(file: File): ValidationResult {
  if (!ALLOWED_TYPES.includes(file.type)) {
    return { valid: false, error: "File type not allowed" };
  }
  if (file.size > MAX_FILE_SIZE) {
    return { valid: false, error: "File too large (max 50MB)" };
  }
  return { valid: true };
}
```

### Server-Side Validation

The server validates files using magic bytes:

```rust
use infer;

pub fn validate_file_type(bytes: &[u8], declared_mime: &str) -> Result<(), ValidationError> {
    // Detect actual type from magic bytes
    let detected = infer::get(bytes)
        .map(|t| t.mime_type())
        .unwrap_or("application/octet-stream");

    // Check if types match (with known variations)
    let types_match = match (detected, declared_mime) {
        (d, s) if d == s => true,
        ("image/jpeg", "image/jpg") | ("image/jpg", "image/jpeg") => true,
        (d, s) if d.starts_with("image/") && s.starts_with("image/") => true,
        _ => false,
    };

    if !types_match {
        return Err(ValidationError::MimeTypeMismatch {
            declared: declared_mime.to_string(),
            detected: detected.to_string(),
        });
    }

    Ok(())
}
```

## Versioning

Each media item can have multiple versions:

```
┌─────────────────────────────────────────────────────────────────┐
│                      Media Versioning                            │
│                                                                  │
│  media_id: abc-123                                               │
│  name: "logo.png"                                                │
│                                                                  │
│  Versions:                                                       │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Version 1 (superseded)                                    │   │
│  │ - Uploaded: 2024-01-01                                    │   │
│  │ - Size: 45KB                                              │   │
│  │ - State: superseded                                       │   │
│  └──────────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Version 2 (active) ← current                              │   │
│  │ - Uploaded: 2024-01-15                                    │   │
│  │ - Size: 52KB                                              │   │
│  │ - State: active                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Version 3 (pending)                                       │   │
│  │ - Uploading...                                            │   │
│  │ - State: pending                                          │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Version States

| State | Description |
|-------|-------------|
| `pending` | Upload initiated but not finalised |
| `active` | Current version displayed to users |
| `superseded` | Previous version, kept for history |
| `failed` | Upload failed, marked for cleanup |

## Thumbnail Generation

Thumbnails are generated asynchronously via background jobs:

```rust
// Job handler for thumbnail generation
pub struct GenerateThumbnailHandler {
    pool: Arc<PgPool>,
    blob_adapter: Arc<dyn BlobAdapter>,
}

impl JobHandler for GenerateThumbnailHandler {
    fn job_type(&self) -> &'static str {
        "media.generate_thumbnail"
    }

    async fn execute(&self, job: &Job) -> Result<(), JobError> {
        let payload: ThumbnailPayload = serde_json::from_value(job.payload.clone())?;

        // Download original image
        let original = self.blob_adapter.get(&payload.blob_key).await?;

        // Generate thumbnail (256x256 max)
        let thumbnail = image::load_from_memory(&original)?
            .thumbnail(256, 256)
            .to_rgb8();

        // Upload thumbnail
        let thumb_key = format!("thumbs/{}", payload.version_id);
        self.blob_adapter.put(&thumb_key, &thumbnail.to_vec()).await?;

        // Create rendition record
        create_rendition(
            &self.pool,
            payload.version_id,
            "thumbnail",
            256,
            256,
            thumb_key,
        ).await?;

        Ok(())
    }
}
```

### Rendition Storage

```sql
CREATE TABLE media.media_renditions (
    id UUID PRIMARY KEY,
    version_id UUID NOT NULL REFERENCES media.media_versions(id),
    rendition_type TEXT NOT NULL,  -- 'thumbnail', 'preview', etc.
    width INTEGER,
    height INTEGER,
    blob_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## Soft Delete & Cleanup

### Soft Delete

Media is soft-deleted first, allowing recovery:

```sql
-- Soft delete
UPDATE media.media
SET deleted_at = NOW()
WHERE id = $1;

-- Restore
UPDATE media.media
SET deleted_at = NULL
WHERE id = $1;

-- Permanent delete (purge)
DELETE FROM media.media WHERE id = $1;
```

### Orphan Cleanup Job

A scheduled job cleans up orphaned media:

```rust
pub struct OrphanMediaCleanupHandler {
    pool: Arc<PgPool>,
}

impl JobHandler for OrphanMediaCleanupHandler {
    fn job_type(&self) -> &'static str {
        "media.cleanup_orphans"
    }

    async fn execute(&self, _job: &Job) -> Result<(), JobError> {
        // Find media with no usages, soft-deleted > 30 days ago
        let orphans = sqlx::query_as::<_, (Uuid,)>(r#"
            SELECT m.id
            FROM media.media m
            WHERE m.deleted_at IS NOT NULL
              AND m.deleted_at < NOW() - INTERVAL '30 days'
              AND NOT EXISTS (
                  SELECT 1 FROM media.media_usages u
                  WHERE u.media_id = m.id
              )
        "#)
        .fetch_all(&*self.pool)
        .await?;

        for (media_id,) in orphans {
            // Purge media and associated blobs
            purge_media(&self.pool, media_id).await?;
        }

        Ok(())
    }
}
```

## API Endpoints

### Media CRUD

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/admin/media` | List media |
| POST | `/v1/admin/media` | Create media record |
| GET | `/v1/admin/media/:id` | Get media details |
| PUT | `/v1/admin/media/:id` | Update media metadata |
| DELETE | `/v1/admin/media/:id` | Purge media |
| POST | `/v1/admin/media/:id/soft-delete` | Soft delete |
| POST | `/v1/admin/media/:id/restore` | Restore |

### Version Management

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/admin/media/:id/versions` | List versions |
| POST | `/v1/admin/media/:id/versions/initiate-upload` | Start upload |
| POST | `/v1/admin/media/:id/versions/:vid/finalise-upload` | Complete upload |
| POST | `/v1/admin/media/:id/versions/:vid/activate` | Set as active |
| DELETE | `/v1/admin/media/:id/versions/:vid` | Delete version |

### Utilities

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/admin/media/check-duplicate` | Check for existing hash |
| POST | `/v1/admin/media:batch-delete` | Batch soft delete |

## Database Schema

### Media Table

```sql
CREATE TABLE media.media (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    alt_text TEXT,
    kind TEXT NOT NULL,       -- 'image', 'document', 'video'
    visibility TEXT NOT NULL, -- 'public', 'private'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
```

### Media Usages Table

```sql
-- Track where media is used (for referential integrity)
CREATE TABLE media.media_usages (
    id UUID PRIMARY KEY,
    media_id UUID NOT NULL REFERENCES media.media(id),
    entity_type TEXT NOT NULL,  -- 'project', 'task', 'user', etc.
    entity_id UUID NOT NULL,
    field_name TEXT NOT NULL,   -- 'thumbnail', 'attachment', etc.
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(media_id, entity_type, entity_id, field_name)
);
```

## Frontend Integration

### Upload Component Pattern

```svelte
<script lang="ts">
  import { mediaCommands, computeFileHash } from "acme-client";

  let uploading = $state(false);
  let progress = $state(0);

  async function handleUpload(file: File) {
    uploading = true;
    progress = 0;

    try {
      // 1. Hash file for deduplication
      const hash = await computeFileHash(file);

      // 2. Check for duplicate
      const dup = await mediaCommands.checkDuplicate(fetch, token, { sha256: hash });
      if (dup.exists) {
        return dup.existingMedia;
      }

      // 3. Create media record
      const media = await mediaCommands.createMedia(fetch, token, {
        name: file.name,
        kind: "image",
        visibility: "public",
      });

      // 4. Initiate upload
      const upload = await mediaCommands.initiateUpload(
        media.id,
        fetch,
        token,
        { mimeType: file.type, fileSize: file.size }
      );

      // 5. Upload to blob storage with progress
      await uploadToBlob(upload.uploadUrl, file, (p) => {
        progress = p;
      });

      // 6. Finalise
      await mediaCommands.finaliseUpload(
        media.id,
        upload.versionId,
        fetch,
        token,
        { sha256: hash }
      );

      return media;
    } finally {
      uploading = false;
    }
  }
</script>
```

### Displaying Media

```svelte
<script lang="ts">
  interface Props {
    media: MediaSummary;
    size?: "thumbnail" | "full";
  }

  let { media, size = "thumbnail" }: Props = $props();

  // Get appropriate URL based on size
  const url = $derived(
    size === "thumbnail"
      ? media.thumbnailUrl ?? media.activeVersion?.url
      : media.activeVersion?.url
  );
</script>

{#if url}
  <img
    src={url}
    alt={media.altText ?? media.name}
    loading="lazy"
  />
{:else}
  <div class="placeholder">No image</div>
{/if}
```
