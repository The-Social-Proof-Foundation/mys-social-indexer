// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::db::DbPool;
use crate::models::profile_events::ProfileEvent;
use crate::schema;

/// Query parameters for fetching profile events
#[derive(Debug, Deserialize)]
pub struct ProfileEventsQuery {
    /// Event type filter (optional)
    #[serde(rename = "event_type")]
    pub event_type: Option<String>,
    
    /// Limit for number of events to return
    #[serde(default = "default_limit")]
    pub limit: i64,
    
    /// Offset for pagination
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

/// Response type for profile events
#[derive(Debug, Serialize)]
pub struct ProfileEventsResponse {
    /// List of profile events
    pub events: Vec<ProfileEvent>,
    
    /// Total count of events (for pagination)
    pub total: i64,
}

/// Handler for getting profile events by profile ID
pub async fn get_profile_events(
    Path(profile_id): Path<String>,
    Query(query): Query<ProfileEventsQuery>,
    State(pool): State<DbPool>,
) -> Result<Json<ProfileEventsResponse>, StatusCode> {
    debug!("Getting profile events for profile_id: {}", profile_id);
    
    let mut conn = pool.get()
        .await
        .map_err(|e| {
            error!("Failed to get database connection: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Build the base query
    let mut query_builder = schema::profile_events::table
        .filter(schema::profile_events::profile_id.eq(&profile_id))
        .into_boxed();
    
    // Apply event type filter if provided
    if let Some(event_type) = &query.event_type {
        query_builder = query_builder.filter(schema::profile_events::event_type.eq(event_type));
    }
    
    // Get total count for pagination - we need to build a separate query since we can't clone BoxedSelectStatement
    let total = schema::profile_events::table
        .filter(schema::profile_events::profile_id.eq(&profile_id))
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to get profile events count: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Get the actual events with limit and offset
    let events = query_builder
        .order_by(schema::profile_events::created_at.desc())
        .limit(query.limit)
        .offset(query.offset)
        .load::<ProfileEvent>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to get profile events: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    debug!("Found {} profile events for profile_id: {}", events.len(), profile_id);
    
    Ok(Json(ProfileEventsResponse { events, total }))
}

/// Get platform membership history for a profile
pub async fn get_platform_memberships(
    Path(profile_id): Path<String>,
    State(pool): State<DbPool>,
) -> Result<Json<ProfileEventsResponse>, StatusCode> {
    debug!("Getting platform memberships for profile_id: {}", profile_id);
    
    let mut conn = pool.get()
        .await
        .map_err(|e| {
            error!("Failed to get database connection: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Query for PlatformJoined and PlatformLeft events
    let query = schema::profile_events::table
        .filter(schema::profile_events::profile_id.eq(&profile_id))
        .filter(
            schema::profile_events::event_type.eq("PlatformJoinedEvent")
            .or(schema::profile_events::event_type.eq("PlatformLeftEvent"))
        )
        .order_by(schema::profile_events::created_at.desc());
    
    // Get total count - rebuilding similar query for count
    let total = schema::profile_events::table
        .filter(schema::profile_events::profile_id.eq(&profile_id))
        .filter(
            schema::profile_events::event_type.eq("PlatformJoinedEvent")
            .or(schema::profile_events::event_type.eq("PlatformLeftEvent"))
        )
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to get platform memberships count: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Get the events
    let events = query
        .load::<ProfileEvent>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to get platform memberships: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    debug!("Found {} platform membership events for profile_id: {}", events.len(), profile_id);
    
    Ok(Json(ProfileEventsResponse { events, total }))
}

/// Get blocking history for a profile
pub async fn get_blocking_history(
    Path(profile_id): Path<String>,
    State(pool): State<DbPool>,
) -> Result<Json<ProfileEventsResponse>, StatusCode> {
    debug!("Getting blocking history for profile_id: {}", profile_id);
    
    let mut conn = pool.get()
        .await
        .map_err(|e| {
            error!("Failed to get database connection: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Query for BlockAdded and BlockRemoved events
    let query = schema::profile_events::table
        .filter(schema::profile_events::profile_id.eq(&profile_id))
        .filter(
            schema::profile_events::event_type.eq("BlockAddedEvent")
            .or(schema::profile_events::event_type.eq("BlockRemovedEvent"))
        )
        .order_by(schema::profile_events::created_at.desc());
    
    // Get total count - rebuilding similar query for count
    let total = schema::profile_events::table
        .filter(schema::profile_events::profile_id.eq(&profile_id))
        .filter(
            schema::profile_events::event_type.eq("BlockAddedEvent")
            .or(schema::profile_events::event_type.eq("BlockRemovedEvent"))
        )
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to get blocking history count: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Get the events
    let events = query
        .load::<ProfileEvent>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to get blocking history: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    debug!("Found {} blocking events for profile_id: {}", events.len(), profile_id);
    
    Ok(Json(ProfileEventsResponse { events, total }))
}