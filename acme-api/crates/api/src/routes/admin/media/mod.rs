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
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use underlay_blob::UploadRequest;
use underlay_http::{context::RequestContext, query::QueryParams, ApiError};
use underlay_jobs::JobConfig;
use underlay_media::storage::version_key;
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
