// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{debug, error};
use serde::{Deserialize};
use chrono::NaiveDateTime;

use crate::db::DbPool;
use crate::models::platform::{Platform, PlatformModerator, PlatformBlockedProfile, PlatformWithDetails};
use crate::schema::{platforms, platform_moderators, platform_blocked_profiles};

#[derive(Debug, Deserialize)]
pub struct PlatformQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub page: Option<i64>,
}

/// Get a list of all platforms with pagination
pub async fn get_platforms(
    State(db_pool): State<DbPool>,
    Query(query): Query<PlatformQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    let page = query.page.unwrap_or(1);
    
    // If page is provided, calculate the offset
    let offset = if page > 1 {
        (page - 1) * limit
    } else {
        offset
    };
    
    debug!("Getting platforms list with limit: {}, offset: {}", limit, offset);
    
    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Database connection error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Database error: {}", e)
                }))
            )
        }
    };
    
    // Get the total count for pagination info
    let total_count = match platforms::table
        .count()
        .get_result::<i64>(&mut conn)
        .await {
        Ok(count) => count,
        Err(_) => 0,
    };
    
    let total_pages = (total_count as f64 / limit as f64).ceil() as i64;
    
    // Query platforms with pagination
    let platforms_result = platforms::table
        .order_by(platforms::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<Platform>(&mut conn)
        .await;
    
    match platforms_result {
        Ok(platforms) => {
            // For each platform, get additional information like moderator count
            let mut platform_details = Vec::with_capacity(platforms.len());
            
            for platform in platforms {
                // Get moderator count
                let moderator_count = platform_moderators::table
                    .filter(platform_moderators::platform_id.eq(&platform.platform_id))
                    .count()
                    .get_result::<i64>(&mut conn)
                    .await
                    .unwrap_or(0);
                
                // Get blocked profiles count
                let blocked_count = platform_blocked_profiles::table
                    .filter(platform_blocked_profiles::platform_id.eq(&platform.platform_id))
                    .filter(platform_blocked_profiles::is_blocked.eq(true))
                    .count()
                    .get_result::<i64>(&mut conn)
                    .await
                    .unwrap_or(0);
                
                // Convert platform_names from JSON to Vec<String>
                let platform_names: Option<Vec<String>> = platform.platform_names
                    .as_ref()
                    .and_then(|json| serde_json::from_value(json.clone()).ok());
                
                // Convert links from JSON to Vec<String>
                let links: Option<Vec<String>> = platform.links
                    .as_ref()
                    .and_then(|json| serde_json::from_value(json.clone()).ok());
                
                // Build response with details
                platform_details.push(PlatformWithDetails {
                    id: platform.id,
                    platform_id: platform.platform_id,
                    name: platform.name,
                    tagline: platform.tagline,
                    description: platform.description,
                    logo: platform.logo,
                    developer_address: platform.developer_address,
                    terms_of_service: platform.terms_of_service,
                    privacy_policy: platform.privacy_policy,
                    platform_names,
                    links,
                    status: platform.status,
                    status_text: PlatformWithDetails::status_to_text(platform.status),
                    release_date: platform.release_date,
                    shutdown_date: platform.shutdown_date,
                    created_at: platform.created_at,
                    updated_at: platform.updated_at,
                    is_approved: platform.is_approved,
                    approval_changed_at: platform.approval_changed_at,
                    approved_by: platform.approved_by.clone(),
                    moderator_count,
                    blocked_profiles_count: blocked_count,
                });
            }
            
            (StatusCode::OK, Json(serde_json::json!({
                "platforms": platform_details,
                "pagination": {
                    "total": total_count,
                    "limit": limit,
                    "offset": offset,
                    "page": page,
                    "total_pages": total_pages
                }
            })))
        },
        Err(e) => {
            error!("Failed to fetch platforms: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch platforms: {}", e)
                }))
            )
        }
    }
}

/// Get a platform by its ID
pub async fn get_platform_by_id(
    State(db_pool): State<DbPool>,
    Path(platform_id): Path<String>,
) -> impl IntoResponse {
    debug!("Getting platform with ID: {}", platform_id);
    
    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Database connection error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Database error: {}", e)
                }))
            )
        }
    };
    
    // Get the platform
    let platform_result = platforms::table
        .filter(platforms::platform_id.eq(&platform_id))
        .first::<Platform>(&mut conn)
        .await;
    
    match platform_result {
        Ok(platform) => {
            // Get moderator count
            let moderator_count = platform_moderators::table
                .filter(platform_moderators::platform_id.eq(&platform.platform_id))
                .count()
                .get_result::<i64>(&mut conn)
                .await
                .unwrap_or(0);
            
            // Get blocked profiles count
            let blocked_count = platform_blocked_profiles::table
                .filter(platform_blocked_profiles::platform_id.eq(&platform.platform_id))
                .filter(platform_blocked_profiles::is_blocked.eq(true))
                .count()
                .get_result::<i64>(&mut conn)
                .await
                .unwrap_or(0);
            
            // Get moderators
            let moderators = platform_moderators::table
                .filter(platform_moderators::platform_id.eq(&platform.platform_id))
                .load::<PlatformModerator>(&mut conn)
                .await
                .unwrap_or_default();
            
            // Convert platform_names from JSON to Vec<String>
            let platform_names: Option<Vec<String>> = platform.platform_names
                .as_ref()
                .and_then(|json| serde_json::from_value(json.clone()).ok());
            
            // Convert links from JSON to Vec<String>
            let links: Option<Vec<String>> = platform.links
                .as_ref()
                .and_then(|json| serde_json::from_value(json.clone()).ok());
            
            // Build response with details
            let platform_details = PlatformWithDetails {
                id: platform.id,
                platform_id: platform.platform_id,
                name: platform.name,
                tagline: platform.tagline,
                description: platform.description,
                logo: platform.logo,
                developer_address: platform.developer_address,
                terms_of_service: platform.terms_of_service,
                privacy_policy: platform.privacy_policy,
                platform_names,
                links,
                status: platform.status,
                status_text: PlatformWithDetails::status_to_text(platform.status),
                release_date: platform.release_date,
                shutdown_date: platform.shutdown_date,
                created_at: platform.created_at,
                updated_at: platform.updated_at,
                is_approved: platform.is_approved,
                approval_changed_at: platform.approval_changed_at,
                approved_by: platform.approved_by.clone(),
                moderator_count,
                blocked_profiles_count: blocked_count,
            };
            
            (StatusCode::OK, Json(serde_json::json!({
                "platform": platform_details,
                "moderators": moderators
            })))
        },
        Err(diesel::result::Error::NotFound) => {
            debug!("Platform not found: {}", platform_id);
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Platform not found"
                }))
            )
        },
        Err(e) => {
            error!("Failed to fetch platform: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch platform: {}", e)
                }))
            )
        }
    }
}

/// Get platform moderators
pub async fn get_platform_moderators(
    State(db_pool): State<DbPool>,
    Path(platform_id): Path<String>,
    Query(query): Query<PlatformQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    let page = query.page.unwrap_or(1);
    
    // If page is provided, calculate the offset
    let offset = if page > 1 {
        (page - 1) * limit
    } else {
        offset
    };
    
    debug!("Getting moderators for platform: {}", platform_id);
    
    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Database connection error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Database error: {}", e)
                }))
            )
        }
    };
    
    // Check if platform exists
    let platform_exists = match platforms::table
        .filter(platforms::platform_id.eq(&platform_id))
        .count()
        .get_result::<i64>(&mut conn)
        .await {
        Ok(count) => count > 0,
        Err(e) => {
            error!("Failed to check platform: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to check platform: {}", e)
                }))
            )
        }
    };
    
    if !platform_exists {
        debug!("Platform not found: {}", platform_id);
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Platform not found"
            }))
        )
    }
    
    // Get the total count for pagination info
    let total_count = match platform_moderators::table
        .filter(platform_moderators::platform_id.eq(&platform_id))
        .count()
        .get_result::<i64>(&mut conn)
        .await {
        Ok(count) => count,
        Err(_) => 0,
    };
    
    let total_pages = (total_count as f64 / limit as f64).ceil() as i64;
    
    // Get moderators with pagination
    let moderators_result = platform_moderators::table
        .filter(platform_moderators::platform_id.eq(&platform_id))
        .order_by(platform_moderators::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<PlatformModerator>(&mut conn)
        .await;
    
    match moderators_result {
        Ok(moderators) => {
            (StatusCode::OK, Json(serde_json::json!({
                "moderators": moderators,
                "pagination": {
                    "total": total_count,
                    "limit": limit,
                    "offset": offset,
                    "page": page,
                    "total_pages": total_pages
                }
            })))
        },
        Err(e) => {
            error!("Failed to fetch moderators: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch moderators: {}", e)
                }))
            )
        }
    }
}

/// Get a list of approved platforms with pagination
pub async fn get_approved_platforms(
    State(db_pool): State<DbPool>,
    Query(query): Query<PlatformQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    let page = query.page.unwrap_or(1);
    
    // If page is provided, calculate the offset
    let offset = if page > 1 {
        (page - 1) * limit
    } else {
        offset
    };
    
    debug!("Getting approved platforms list with limit: {}, offset: {}", limit, offset);
    
    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Database connection error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Database error: {}", e)
                }))
            )
        }
    };
    
    // Get the total count for pagination info (only approved platforms)
    let total_count = match platforms::table
        .filter(platforms::is_approved.eq(true))
        .count()
        .get_result::<i64>(&mut conn)
        .await {
        Ok(count) => count,
        Err(_) => 0,
    };
    
    let total_pages = (total_count as f64 / limit as f64).ceil() as i64;
    
    // Query platforms with pagination, filtered by approval status
    let platforms_result = platforms::table
        .filter(platforms::is_approved.eq(true))
        .order_by(platforms::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<Platform>(&mut conn)
        .await;
    
    match platforms_result {
        Ok(platforms) => {
            // For each platform, get additional information like moderator count
            let mut platform_details = Vec::with_capacity(platforms.len());
            
            for platform in platforms {
                // Get moderator count
                let moderator_count = platform_moderators::table
                    .filter(platform_moderators::platform_id.eq(&platform.platform_id))
                    .count()
                    .get_result::<i64>(&mut conn)
                    .await
                    .unwrap_or(0);
                
                // Get blocked profiles count
                let blocked_count = platform_blocked_profiles::table
                    .filter(platform_blocked_profiles::platform_id.eq(&platform.platform_id))
                    .filter(platform_blocked_profiles::is_blocked.eq(true))
                    .count()
                    .get_result::<i64>(&mut conn)
                    .await
                    .unwrap_or(0);
                
                // Convert platform_names from JSON to Vec<String>
                let platform_names: Option<Vec<String>> = platform.platform_names
                    .as_ref()
                    .and_then(|json| serde_json::from_value(json.clone()).ok());
                
                // Convert links from JSON to Vec<String>
                let links: Option<Vec<String>> = platform.links
                    .as_ref()
                    .and_then(|json| serde_json::from_value(json.clone()).ok());
                
                // Build response with details
                platform_details.push(PlatformWithDetails {
                    id: platform.id,
                    platform_id: platform.platform_id,
                    name: platform.name,
                    tagline: platform.tagline,
                    description: platform.description,
                    logo: platform.logo,
                    developer_address: platform.developer_address,
                    terms_of_service: platform.terms_of_service,
                    privacy_policy: platform.privacy_policy,
                    platform_names,
                    links,
                    status: platform.status,
                    status_text: PlatformWithDetails::status_to_text(platform.status),
                    release_date: platform.release_date,
                    shutdown_date: platform.shutdown_date,
                    created_at: platform.created_at,
                    updated_at: platform.updated_at,
                    is_approved: platform.is_approved,
                    approval_changed_at: platform.approval_changed_at,
                    approved_by: platform.approved_by.clone(),
                    moderator_count,
                    blocked_profiles_count: blocked_count,
                });
            }
            
            (StatusCode::OK, Json(serde_json::json!({
                "platforms": platform_details,
                "pagination": {
                    "total": total_count,
                    "limit": limit,
                    "offset": offset,
                    "page": page,
                    "total_pages": total_pages
                }
            })))
        },
        Err(e) => {
            error!("Failed to fetch approved platforms: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch approved platforms: {}", e)
                }))
            )
        }
    }
}

/// Get the approval status of a specific platform
pub async fn get_platform_approval_status(
    State(db_pool): State<DbPool>,
    Path(platform_id): Path<String>,
) -> impl IntoResponse {
    debug!("Getting approval status for platform: {}", platform_id);
    
    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Database connection error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Database error: {}", e)
                }))
            )
        }
    };
    
    // Get the platform
    let platform_result = platforms::table
        .filter(platforms::platform_id.eq(&platform_id))
        .select((
            platforms::is_approved,
            platforms::approval_changed_at,
            platforms::approved_by
        ))
        .first::<(bool, Option<NaiveDateTime>, Option<String>)>(&mut conn)
        .await;
    
    match platform_result {
        Ok((is_approved, approval_changed_at, approved_by)) => {
            (StatusCode::OK, Json(serde_json::json!({
                "platform_id": platform_id,
                "is_approved": is_approved,
                "approval_changed_at": approval_changed_at,
                "approved_by": approved_by
            })))
        },
        Err(diesel::result::Error::NotFound) => {
            debug!("Platform not found: {}", platform_id);
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Platform not found"
                }))
            )
        },
        Err(e) => {
            error!("Failed to fetch platform approval status: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch platform approval status: {}", e)
                }))
            )
        }
    }
}

pub async fn get_platform_blocked_profiles(
    State(db_pool): State<DbPool>,
    Path(platform_id): Path<String>,
    Query(query): Query<PlatformQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    let page = query.page.unwrap_or(1);
    
    // If page is provided, calculate the offset
    let offset = if page > 1 {
        (page - 1) * limit
    } else {
        offset
    };
    
    debug!("Getting blocked profiles for platform: {}", platform_id);
    
    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Database connection error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Database error: {}", e)
                }))
            )
        }
    };
    
    // Check if platform exists
    let platform_exists = match platforms::table
        .filter(platforms::platform_id.eq(&platform_id))
        .count()
        .get_result::<i64>(&mut conn)
        .await {
        Ok(count) => count > 0,
        Err(e) => {
            error!("Failed to check platform: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to check platform: {}", e)
                }))
            )
        }
    };
    
    if !platform_exists {
        debug!("Platform not found: {}", platform_id);
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Platform not found"
            }))
        )
    }
    
    // Get the total count for pagination info
    let total_count = match platform_blocked_profiles::table
        .filter(platform_blocked_profiles::platform_id.eq(&platform_id))
        .filter(platform_blocked_profiles::is_blocked.eq(true))
        .count()
        .get_result::<i64>(&mut conn)
        .await {
        Ok(count) => count,
        Err(_) => 0,
    };
    
    let total_pages = (total_count as f64 / limit as f64).ceil() as i64;
    
    // Get blocked profiles with pagination
    let blocked_result = platform_blocked_profiles::table
        .filter(platform_blocked_profiles::platform_id.eq(&platform_id))
        .filter(platform_blocked_profiles::is_blocked.eq(true))
        .order_by(platform_blocked_profiles::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<PlatformBlockedProfile>(&mut conn)
        .await;
    
    match blocked_result {
        Ok(blocked) => {
            (StatusCode::OK, Json(serde_json::json!({
                "blocked_profiles": blocked,
                "pagination": {
                    "total": total_count,
                    "limit": limit,
                    "offset": offset,
                    "page": page,
                    "total_pages": total_pages
                }
            })))
        },
        Err(e) => {
            error!("Failed to fetch blocked profiles: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch blocked profiles: {}", e)
                }))
            )
        }
    }
}