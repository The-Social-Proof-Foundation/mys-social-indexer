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
use serde::{Deserialize};

use crate::db::DbPool;
use crate::models::Profile;
use crate::schema::profiles;

#[derive(Debug, Deserialize)]
pub struct ProfileQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Get a list of profiles with pagination
pub async fn get_profiles(
    State(db_pool): State<DbPool>,
    Query(query): Query<ProfileQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    
    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Database error: {}", e)
                }))
            )
        }
    };
    
    let profiles_result = profiles::table
        .limit(limit)
        .offset(offset)
        .load::<Profile>(&mut conn)
        .await;
    
    match profiles_result {
        Ok(profiles) => (StatusCode::OK, Json(serde_json::to_value(profiles).unwrap_or_default())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to fetch profiles: {}", e)
            }))
        )
    }
}

/// Get a profile by address
pub async fn get_profile_by_address(
    State(db_pool): State<DbPool>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Database error: {}", e)
                }))
            )
        }
    };
    
    let profile_result = profiles::table
        .filter(profiles::owner_address.eq(address))
        .first::<Profile>(&mut conn)
        .await;
    
    match profile_result {
        Ok(profile) => (StatusCode::OK, Json(serde_json::to_value(profile).unwrap_or_default())),
        Err(diesel::result::Error::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Profile not found"
            }))
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to fetch profile: {}", e)
            }))
        )
    }
}

/// Get a profile by username
pub async fn get_profile_by_username(
    State(db_pool): State<DbPool>,
    Path(username): Path<String>,
) -> impl IntoResponse {
    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Database error: {}", e)
                }))
            )
        }
    };
    
    let profile_result = profiles::table
        .filter(profiles::username.eq(username))
        .first::<Profile>(&mut conn)
        .await;
    
    match profile_result {
        Ok(profile) => (StatusCode::OK, Json(serde_json::to_value(profile).unwrap_or_default())),
        Err(diesel::result::Error::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Profile not found"
            }))
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to fetch profile: {}", e)
            }))
        )
    }
}