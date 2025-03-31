// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

pub mod profile_events;

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;

// Event type prefixes for each module
pub const MODULE_PREFIX_PROFILE: &str = "0x";  // This will be expanded with the actual address in code

/// Parse an event from blockchain JSON
pub fn parse_event<T: DeserializeOwned>(json_value: &Value) -> Result<T> {
    // For debugging, print the event structure
    tracing::debug!("Parsing event JSON: {}", json_value);
    
    // Try direct deserialization first
    if let Ok(result) = serde_json::from_value::<T>(json_value.clone()) {
        return Ok(result);
    }
    
    // If direct parse fails, try different JSON paths that might contain our data
    let parsed_json = if let Value::Object(map) = json_value {
        // Try common patterns
        if let Some(value) = map.get("value") {
            value.clone()
        } else if let Some(data) = map.get("data") {
            data.clone()
        } else if let Some(parsed) = map.get("parsed_json") {
            parsed.clone()
        } else {
            // Just try the whole object
            json_value.clone()
        }
    } else {
        // Just try the raw value
        json_value.clone()
    };

    serde_json::from_value(parsed_json)
        .map_err(|e| anyhow!("Failed to parse event: {}", e))
}