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

use crate::db::DbPool;
use crate::models::social_graph::{FollowDetail, FollowsQuery};
use crate::schema::{social_graph_relationships, profiles};

/// Get a list of profiles that a user is following
pub async fn get_following(
    State(db_pool): State<DbPool>,
    Path(profile_id): Path<String>,
    Query(query): Query<FollowsQuery>,
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
    
    debug!("Getting following for profile_id: {}, limit: {}, offset: {}", profile_id, limit, offset);
    
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
    
    // First verify the profile exists using profile_id
    let profile_exists = match profiles::table
        .filter(profiles::profile_id.eq(&profile_id))
        .count()
        .get_result::<i64>(&mut conn)
        .await {
        Ok(count) => count > 0,
        Err(e) => {
            error!("Failed to check profile: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to check profile: {}", e)
                }))
            )
        }
    };
    
    if !profile_exists {
        debug!("Profile not found with profile_id: {}", profile_id);
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Profile not found"
            }))
        )
    }
    
    // Get following relationships and join with profiles to get details
    // Now using profile_id instead of owner_address
    let following_query = social_graph_relationships::table
        .filter(social_graph_relationships::follower_address.eq(&profile_id))
        .inner_join(profiles::table.on(
            diesel::dsl::sql::<diesel::sql_types::Bool>("profiles.profile_id = social_graph_relationships.following_address")
        ))
        .select((
            profiles::id,
            profiles::profile_id,
            profiles::owner_address,
            profiles::username,
            profiles::display_name.nullable(),
            profiles::profile_photo.nullable(),
            profiles::bio.nullable(),
            profiles::website.nullable(),
            social_graph_relationships::created_at,
        ))
        .limit(limit)
        .offset(offset)
        .order_by(social_graph_relationships::created_at.desc());
        
    let following_result = following_query
        .load::<(i32, Option<String>, String, String, Option<String>, Option<String>, Option<String>, Option<String>, chrono::NaiveDateTime)>(&mut conn)
        .await;
        
    // Also get the total count for pagination info
    let total_count = match social_graph_relationships::table
        .filter(social_graph_relationships::follower_address.eq(&profile_id))
        .count()
        .get_result::<i64>(&mut conn)
        .await {
        Ok(count) => count,
        Err(_) => 0,
    };
    
    let total_pages = (total_count as f64 / limit as f64).ceil() as i64;
        
    match following_result {
        Ok(follows) => {
            // Map to FollowDetail struct
            let follows_detail: Vec<FollowDetail> = follows
                .into_iter()
                .map(|(id, profile_id, owner_address, username, display_name, profile_photo, bio, website, followed_at)| {
                    FollowDetail {
                        id,
                        profile_id,
                        owner_address,
                        username,
                        display_name,
                        profile_photo,
                        bio,
                        website,
                        followed_at,
                    }
                })
                .collect();
                
            (StatusCode::OK, Json(serde_json::json!({
                "profiles": follows_detail,
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
            error!("Failed to fetch following profiles: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch following: {}", e)
                }))
            )
        }
    }
}

/// Get a list of profiles that follow a user
pub async fn get_followers(
    State(db_pool): State<DbPool>,
    Path(profile_id): Path<String>,
    Query(query): Query<FollowsQuery>,
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
    
    debug!("Getting followers for profile_id: {}, limit: {}, offset: {}", profile_id, limit, offset);
    
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
    
    // First verify the profile exists using profile_id
    let profile_exists = match profiles::table
        .filter(profiles::profile_id.eq(&profile_id))
        .count()
        .get_result::<i64>(&mut conn)
        .await {
        Ok(count) => count > 0,
        Err(e) => {
            error!("Failed to check profile: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to check profile: {}", e)
                }))
            )
        }
    };
    
    if !profile_exists {
        debug!("Profile not found with profile_id: {}", profile_id);
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Profile not found"
            }))
        )
    }
    
    // Get follower relationships and join with profiles to get details
    // Now using profile_id instead of owner_address
    let followers_query = social_graph_relationships::table
        .filter(social_graph_relationships::following_address.eq(&profile_id))
        .inner_join(profiles::table.on(
            diesel::dsl::sql::<diesel::sql_types::Bool>("profiles.profile_id = social_graph_relationships.follower_address")
        ))
        .select((
            profiles::id,
            profiles::profile_id,
            profiles::owner_address,
            profiles::username,
            profiles::display_name.nullable(),
            profiles::profile_photo.nullable(),
            profiles::bio.nullable(),
            profiles::website.nullable(),
            social_graph_relationships::created_at,
        ))
        .limit(limit)
        .offset(offset)
        .order_by(social_graph_relationships::created_at.desc());
        
    let followers_result = followers_query
        .load::<(i32, Option<String>, String, String, Option<String>, Option<String>, Option<String>, Option<String>, chrono::NaiveDateTime)>(&mut conn)
        .await;
        
    // Also get the total count for pagination info
    let total_count = match social_graph_relationships::table
        .filter(social_graph_relationships::following_address.eq(&profile_id))
        .count()
        .get_result::<i64>(&mut conn)
        .await {
        Ok(count) => count,
        Err(_) => 0,
    };
    
    let total_pages = (total_count as f64 / limit as f64).ceil() as i64;
        
    match followers_result {
        Ok(follows) => {
            // Map to FollowDetail struct
            let follows_detail: Vec<FollowDetail> = follows
                .into_iter()
                .map(|(id, profile_id, owner_address, username, display_name, profile_photo, bio, website, followed_at)| {
                    FollowDetail {
                        id,
                        profile_id,
                        owner_address,
                        username,
                        display_name,
                        profile_photo,
                        bio,
                        website,
                        followed_at,
                    }
                })
                .collect();
                
            (StatusCode::OK, Json(serde_json::json!({
                "profiles": follows_detail,
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
            error!("Failed to fetch follower profiles: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch followers: {}", e)
                }))
            )
        }
    }
}

/// Check if a user is following another user
pub async fn check_following(
    State(db_pool): State<DbPool>,
    Path((follower_profile_id, following_profile_id)): Path<(String, String)>,
) -> impl IntoResponse {
    debug!("Checking if profile {} follows profile {}", follower_profile_id, following_profile_id);
    
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
    
    // Check if both profiles exist using profile_id
    let follower_exists = match profiles::table
        .filter(profiles::profile_id.eq(&follower_profile_id))
        .count()
        .get_result::<i64>(&mut conn)
        .await {
        Ok(count) => count > 0,
        Err(e) => {
            error!("Failed to check follower profile: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to check follower profile: {}", e),
                    "is_following": false
                }))
            )
        }
    };
    
    if !follower_exists {
        debug!("Follower profile not found: {}", follower_profile_id);
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Follower profile not found",
                "is_following": false
            }))
        )
    }
    
    let following_exists = match profiles::table
        .filter(profiles::profile_id.eq(&following_profile_id))
        .count()
        .get_result::<i64>(&mut conn)
        .await {
        Ok(count) => count > 0,
        Err(e) => {
            error!("Failed to check following profile: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to check following profile: {}", e),
                    "is_following": false
                }))
            )
        }
    };
    
    if !following_exists {
        debug!("Following profile not found: {}", following_profile_id);
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Following profile not found",
                "is_following": false
            }))
        )
    }
    
    // Check if a relationship exists using profile_id
    let relationship_exists = social_graph_relationships::table
        .filter(social_graph_relationships::follower_address.eq(&follower_profile_id))
        .filter(social_graph_relationships::following_address.eq(&following_profile_id))
        .count()
        .get_result::<i64>(&mut conn)
        .await;
        
    match relationship_exists {
        Ok(count) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "is_following": count > 0
            }))
        ),
        Err(e) => {
            error!("Failed to check following status: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to check follow status: {}", e),
                    "is_following": false
                }))
            )
        }
    }
}

/// Get stats for a profile (followers count, following count)
pub async fn get_follow_stats(
    State(db_pool): State<DbPool>,
    Path(profile_id): Path<String>,
) -> impl IntoResponse {
    debug!("Getting follow stats for profile_id: {}", profile_id);
    
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
    
    // Get profile stats from the profiles table using profile_id
    let profile_result = profiles::table
        .filter(profiles::profile_id.eq(&profile_id))
        .select((
            profiles::followers_count, 
            profiles::following_count,
            profiles::username,
            profiles::display_name.nullable(),
            profiles::profile_photo.nullable()
        ))
        .first::<(i32, i32, String, Option<String>, Option<String>)>(&mut conn)
        .await;
        
    match profile_result {
        Ok((followers, following, username, display_name, profile_photo)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "profile_id": profile_id,
                "username": username,
                "display_name": display_name,
                "profile_photo": profile_photo,
                "followers_count": followers,
                "following_count": following
            }))
        ),
        Err(diesel::result::Error::NotFound) => {
            debug!("Profile not found with profile_id: {}", profile_id);
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Profile not found"
                }))
            )
        },
        Err(e) => {
            error!("Failed to fetch profile stats: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch profile stats: {}", e)
                }))
            )
        }
    }
}