//! Media Library admin routes.
//!
//! Provides admin endpoints for managing media items, versions, and uploads.

mod batch;
mod crud;
mod upload;
mod usage;
mod versions;

pub use batch::*;
pub use crud::*;
pub use upload::*;
pub use usage::*;
pub use versions::*;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use underlay_blob::{BlobAdapterObjectKeyExt, UploadRequest};
use underlay_http::{ApiError, context::RequestContext, query::QueryParams};
use underlay_jobs::JobConfig;
use underlay_media::storage::version_object_key;
use uuid::Uuid;
use validator::Validate;

use acme_core::Uuid as AcmeUuid;
use acme_db::{activity, media};

use crate::dto::media::{
    CheckDuplicateRequest, CheckDuplicateResponse, CreateMediaRequest, FinaliseUploadRequest,
    FinaliseUploadResponse, InitiateUploadRequest, InitiateUploadResponse, MediaDetailDto,
    MediaSummaryDto, MediaUsageDto, MediaVersionDto, UpdateMediaRequest,
};
use crate::state::{AdminUser, AppState};
