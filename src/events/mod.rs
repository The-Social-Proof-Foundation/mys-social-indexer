// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

pub mod profile_events;
pub mod profile_event_types;
pub mod social_graph_events;
pub mod platform_events;
pub mod blocking_events;

// Re-export all profile events
pub use profile_events::{
    ProfileCreatedEvent,
    ProfileUpdatedEvent,
    UsernameUpdatedEvent,
    UsernameRegisteredEvent,
    ProfileFollowEvent,
    ProfileJoinedPlatformEvent,
};

// Re-export profile event types
pub use profile_event_types::{
    ProfileEventType,
    BlockAddedEvent,
    BlockRemovedEvent,
    PlatformJoinedEvent,
    PlatformLeftEvent,
};

// Re-export platform events
pub use crate::models::platform::{
    PlatformCreatedEvent,
    PlatformUpdatedEvent,
    PlatformApprovalChangedEvent,
    ModeratorAddedEvent,
    ModeratorRemovedEvent,
    // These are also defined in blocking models, so use those instead
    // PlatformBlockedProfileEvent,
    // PlatformUnblockedProfileEvent,
};

// Re-export social graph events
pub use social_graph_events::{
    FollowEvent,
    UnfollowEvent,
};

// Re-export blocking events
pub use crate::models::blocking::{
    // Block events
    UserBlockEvent,
    UserUnblockEvent,
    // Platform events
    PlatformBlockedProfileEvent,
    PlatformUnblockedProfileEvent,
};

// BlockListCreatedEvent
pub use crate::events::blocking_events::BlockListCreatedEvent;

// Define placeholder event types for other modules
// These should be moved to their own module files when implemented
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestPlatformCreatedEvent {
    pub platform_id: String,
    pub name: String,
    pub description: String,
    pub creator_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContentCreatedEvent {
    pub content_id: String,
    pub creator_id: String,
    pub platform_id: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContentInteractionEvent {
    pub content_id: String,
    pub profile_id: String,
    pub interaction_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityBlockedEvent {
    pub blocker_id: String,
    pub blocked_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IPRegisteredEvent {
    pub ip_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LicenseGrantedEvent {
    pub license_id: String,
    pub ip_id: String,
    pub payment_amount: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProofCreatedEvent {
    pub proof_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeeModelCreatedEvent {
    pub fee_model_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeesDistributedEvent {
    pub fee_model_id: String,
    pub total_fee_amount: u64,
}

// Implementation traits will be properly implemented when needed
// Currently stubbed out to avoid compilation errors

/*
// For PlatformCreatedEvent
impl PlatformCreatedEvent {
    pub fn into_model(&self) -> Result<()> {
        // Placeholder implementation
        unimplemented!("PlatformCreatedEvent::into_model() not implemented yet")
    }
}

// For ContentCreatedEvent
impl ContentCreatedEvent {
    pub fn into_model(&self) -> Result<()> {
        // Placeholder implementation
        unimplemented!("ContentCreatedEvent::into_model() not implemented yet")
    }
}

// For ContentInteractionEvent
impl ContentInteractionEvent {
    pub fn into_model(&self) -> Result<()> {
        // Placeholder implementation
        unimplemented!("ContentInteractionEvent::into_model() not implemented yet")
    }
}

// For EntityBlockedEvent
impl EntityBlockedEvent {
    pub fn into_model(&self) -> Result<()> {
        // Placeholder implementation
        unimplemented!("EntityBlockedEvent::into_model() not implemented yet")
    }
}

// For IPRegisteredEvent
impl IPRegisteredEvent {
    pub fn into_model(&self, _content_id: Option<String>, _creator_id: Option<String>) -> Result<()> {
        // Placeholder implementation
        unimplemented!("IPRegisteredEvent::into_model() not implemented yet")
    }
}

// For LicenseGrantedEvent
impl LicenseGrantedEvent {
    pub fn into_model(&self, _licensee_id: Option<String>) -> Result<()> {
        // Placeholder implementation
        unimplemented!("LicenseGrantedEvent::into_model() not implemented yet")
    }
}

// For FeesDistributedEvent
impl FeesDistributedEvent {
    pub fn into_model(&self) -> Result<()> {
        // Placeholder implementation
        unimplemented!("FeesDistributedEvent::into_model() not implemented yet")
    }
}
*/

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

// Event type prefixes for each module - all use the same MySocial package address
use crate::DEFAULT_MYSOCIAL_PACKAGE_ADDRESS;

// Helper macro to create module prefixes using the main package address
macro_rules! module_prefix {
    () => { DEFAULT_MYSOCIAL_PACKAGE_ADDRESS };
}

// All modules are in the same package - using a macro means we only need to update one place
pub const MODULE_PREFIX_PROFILE: &str = module_prefix!();
pub const MODULE_PREFIX_PLATFORM: &str = module_prefix!();
pub const MODULE_PREFIX_CONTENT: &str = module_prefix!();
pub const MODULE_PREFIX_BLOCK_LIST: &str = module_prefix!();
pub const MODULE_PREFIX_MY_IP: &str = module_prefix!();
pub const MODULE_PREFIX_FEE_DISTRIBUTION: &str = module_prefix!();
pub const MODULE_PREFIX_SOCIAL_GRAPH: &str = module_prefix!();

/// Parse an event from blockchain JSON
pub fn parse_event<T: DeserializeOwned>(json_value: &Value) -> Result<T> {
    // Print detailed event structure for debugging
    tracing::info!("Parsing event JSON: {}", serde_json::to_string_pretty(json_value).unwrap_or_default());
    
    // Check if we're parsing a profile-related event
    let event_type = std::any::type_name::<T>();
    let is_username_event = event_type.contains("UsernameRegisteredEvent") || 
                            event_type.contains("UsernameUpdatedEvent");
    let is_profile_event = event_type.contains("ProfileCreatedEvent") || 
                           event_type.contains("ProfileUpdatedEvent") ||
                           // PrivateDataUpdatedEvent removed
                           is_username_event;
                           
    // Extra logging for username events
    if is_username_event {
        tracing::info!("Parsing a username event: {}", event_type);
        tracing::info!("Raw event JSON: {}", serde_json::to_string_pretty(json_value).unwrap_or_default());
    }
    
    if is_profile_event {
        tracing::info!("Attempting to parse a profile event: {}", event_type);
        
        // For debugging, create a map with the fields we can find
        let mut extracted_fields = serde_json::Map::new();
        
        // Handle object and content nesting
        if let Value::Object(obj) = json_value {
            // Log all the top-level keys
            tracing::info!("Top-level keys: {:?}", obj.keys().collect::<Vec<_>>());
            
            // Look for the fields structure that contains the data
            let fields = if let Some(fields_val) = obj.get("fields") {
                if let Some(fields_obj) = fields_val.as_object() {
                    Some(fields_obj)
                } else {
                    None
                }
            } else if let Some(content) = obj.get("content") {
                // Try to look in content section for fields
                if let Some(content_obj) = content.as_object() {
                    if let Some(fields_val) = content_obj.get("fields") {
                        fields_val.as_object()
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            
            if let Some(fields) = fields {
                tracing::info!("Found fields: {}", serde_json::to_string_pretty(fields).unwrap_or_default());
                
                // Helper function to extract a value from an object with consistent approach
                fn extract_object_value(obj: &serde_json::Map<String, Value>) -> Option<Value> {
                    // Check for URL field (used for profile_photo and cover_photo)
                    if let Some(url) = obj.get("url") {
                        tracing::info!("    - Found URL field: {}", url);
                        return Some(url.clone());
                    }
                    
                    // Check for string field (used for display_name and bio)
                    if let Some(s) = obj.get("string") {
                        tracing::info!("    - Found string field: {}", s);
                        return Some(s.clone());
                    }
                    
                    // For Move Option<String> structure (nested inside vec array)
                    if let Some(vec_field) = obj.get("vec") {
                        tracing::info!("    - Found vec field: {}", vec_field);
                        if let Some(vec_array) = vec_field.as_array() {
                            if !vec_array.is_empty() {
                                // Try to extract the first element's string value
                                if let Some(first_item) = vec_array.get(0) {
                                    if let Some(string_obj) = first_item.get("string") {
                                        tracing::info!("    - Found string in vec: {}", string_obj);
                                        return Some(string_obj.clone());
                                    } else if first_item.is_string() {
                                        tracing::info!("    - Found direct string in vec: {}", first_item);
                                        return Some(first_item.clone());
                                    } else if first_item.is_object() {
                                        tracing::info!("    - Found object in vec, recursively processing");
                                        if let Some(inner_obj) = first_item.as_object() {
                                            return extract_object_value(inner_obj);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Fall back to the value itself as a string if nothing else works
                    Some(Value::String(serde_json::to_string(obj).unwrap_or_default()))
                }
                
                // Function to extract field value consistently
                let mut extract_field = |field_name: &str, target_name: &str| {
                    if let Some(value) = fields.get(field_name) {
                        tracing::info!("Processing field '{}': {}", field_name, value);
                        
                        if value.is_string() {
                            // Direct string value
                            tracing::info!("  - Direct string value found");
                            extracted_fields.insert(target_name.to_string(), value.clone());
                            true
                        } else if value.is_object() {
                            // Try to extract from object
                            tracing::info!("  - Object value found, looking for string representation");
                            
                            if let Some(obj) = value.as_object() {
                                if let Some(extracted) = extract_object_value(obj) {
                                    extracted_fields.insert(target_name.to_string(), extracted);
                                    return true;
                                }
                            }
                            
                            // If all else fails, use the entire object as a string
                            tracing::info!("  - Using stringified object as fallback");
                            extracted_fields.insert(target_name.to_string(), 
                                Value::String(serde_json::to_string(value).unwrap_or_default()));
                            true
                        } else if value.is_boolean() {
                            // Boolean value (for flags)
                            tracing::info!("  - Boolean value found");
                            extracted_fields.insert(target_name.to_string(), value.clone());
                            true
                        } else if value.is_number() {
                            // Number value (for timestamps)
                            tracing::info!("  - Number value found");
                            extracted_fields.insert(target_name.to_string(), value.clone());
                            true
                        } else if value.is_array() {
                            // Array value (might contain what we need)
                            tracing::info!("  - Array value found, checking first element");
                            if let Some(array) = value.as_array() {
                                if !array.is_empty() {
                                    if let Some(first) = array.get(0) {
                                        // Try to get string or url from the first element
                                        if first.is_string() {
                                            extracted_fields.insert(target_name.to_string(), first.clone());
                                            return true;
                                        } else if first.is_object() {
                                            if let Some(obj) = first.as_object() {
                                                if let Some(extracted) = extract_object_value(obj) {
                                                    extracted_fields.insert(target_name.to_string(), extracted);
                                                    return true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Fallback - use the entire array as a string
                            extracted_fields.insert(target_name.to_string(), 
                                Value::String(serde_json::to_string(value).unwrap_or_default()));
                            true
                        } else {
                            // Other value types
                            tracing::info!("  - Other value type, using as-is");
                            extracted_fields.insert(target_name.to_string(), value.clone());
                            true
                        }
                    } else if field_name == "bio" && fields.get("display_name").is_some() {
                        // Special case: if bio is missing but display_name exists, look for bio in dynamic_fields
                        if let Some(dynamic_fields) = fields.get("id").and_then(|v| v.get("dynamic_fields")) {
                            tracing::info!("  - Looking for bio in dynamic_fields");
                            if let Some(dynamic_array) = dynamic_fields.as_array() {
                                for field in dynamic_array {
                                    if let Some(name) = field.get("name").and_then(|n| n.as_str()) {
                                        if name == "bio" || name.contains("bio") {
                                            if let Some(value) = field.get("value") {
                                                tracing::info!("  - Found bio in dynamic_fields: {}", value);
                                                extracted_fields.insert(target_name.to_string(), value.clone());
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        false
                    } else {
                        false
                    }
                };
                
                // Define field mappings - use the same approach for ALL fields
                let field_mappings = [
                    // Core fields
                    ("id", "profile_id"),
                    ("profile_id", "profile_id"),
                    ("owner", "owner_address"),
                    ("owner_address", "owner_address"),
                    ("display_name", "display_name"),
                    
                    // Text content fields 
                    ("bio", "bio"),
                    
                    // URL fields 
                    ("profile_picture", "profile_photo"),
                    ("profile_photo", "profile_photo"), // Try both names
                    ("cover_photo", "cover_photo"),
                    ("cover_url", "cover_photo"),  // Try both names
                    
                    // Boolean fields
                    ("has_profile_picture", "has_profile_picture"),
                    ("has_cover_photo", "has_cover_photo"),
                    
                    // Timestamp
                    ("created_at", "created_at"),
                    ("updated_at", "updated_at"),
                    ("registered_at", "registered_at"),
                    ("expires_at", "expires_at"),
                    
                    // Username fields
                    ("username", "username"),
                    ("old_username", "old_username"),
                    ("new_username", "new_username"),
                ];
                
                // Process each field mapping consistently
                for (source, target) in field_mappings {
                    if extract_field(source, target) {
                        tracing::info!("Successfully extracted '{}' as '{}'", source, target);
                    } else {
                        tracing::info!("Field '{}' not found", source);
                    }
                }
                
                // Special handling for URLs when we have the "has_X" flags but no URL values
                if let Some(has_profile_pic) = extracted_fields.get("has_profile_picture") {
                    if has_profile_pic.as_bool().unwrap_or(false) && !extracted_fields.contains_key("profile_photo") {
                        // Try to find profile photo in any field that might have it
                        for (k, v) in fields.iter() {
                            if k.contains("profile") && k.contains("pic") || k.contains("photo") || k.contains("avatar") {
                                tracing::info!("Found potential profile photo field '{}': {}", k, v);
                                if let Some(obj) = v.as_object() {
                                    if let Some(extracted) = extract_object_value(obj) {
                                        extracted_fields.insert("profile_photo".to_string(), extracted);
                                        break;
                                    }
                                } else if v.is_string() {
                                    extracted_fields.insert("profile_photo".to_string(), v.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
                
                if let Some(has_cover) = extracted_fields.get("has_cover_photo") {
                    if has_cover.as_bool().unwrap_or(false) && !extracted_fields.contains_key("cover_photo") {
                        // Try to find cover photo in any field that might have it
                        for (k, v) in fields.iter() {
                            if k.contains("cover") {
                                tracing::info!("Found potential cover photo field '{}': {}", k, v);
                                if let Some(obj) = v.as_object() {
                                    if let Some(extracted) = extract_object_value(obj) {
                                        extracted_fields.insert("cover_photo".to_string(), extracted);
                                        break;
                                    }
                                } else if v.is_string() {
                                    extracted_fields.insert("cover_photo".to_string(), v.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
                
                // Parse bio similarly to display_name if not already found
                if !extracted_fields.contains_key("bio") && extracted_fields.contains_key("display_name") {
                    for (k, v) in fields.iter() {
                        if k == "bio" || k.contains("bio") || k.contains("description") {
                            tracing::info!("Found potential bio field '{}': {}", k, v);
                            if let Some(obj) = v.as_object() {
                                if let Some(extracted) = extract_object_value(obj) {
                                    extracted_fields.insert("bio".to_string(), extracted);
                                    break;
                                }
                            } else if v.is_string() {
                                extracted_fields.insert("bio".to_string(), v.clone());
                                break;
                            }
                        }
                    }
                }
                
                // Additional username extraction (might be in dynamic fields)
                // Extract username-related fields (for UsernameRegisteredEvent)
                if event_type.contains("UsernameRegisteredEvent") {
                    tracing::info!("Extracting fields for UsernameRegisteredEvent");
                    tracing::info!("All available fields:");
                    for (k, v) in fields.iter() {
                        tracing::info!("Field '{}': {}", k, v);
                    }
                    
                    // Username - CRITICAL: Must be found for username events
                    if !extracted_fields.contains_key("username") {
                        if let Some(username) = fields.get("username") {
                            tracing::info!("Found primary username field: {}", username);
                            if username.is_string() {
                                extracted_fields.insert("username".to_string(), username.clone());
                            } else if let Some(username_str) = username.as_str() {
                                extracted_fields.insert("username".to_string(), Value::String(username_str.to_string()));
                            } else if let Some(obj) = username.as_object() {
                                if let Some(str_val) = obj.get("string") {
                                    extracted_fields.insert("username".to_string(), str_val.clone());
                                } else {
                                    // Try to convert the object to a string
                                    let username_str = serde_json::to_string(username)
                                        .unwrap_or_else(|_| "".to_string());
                                    extracted_fields.insert("username".to_string(), Value::String(username_str));
                                }
                            } else {
                                // Try to convert to string as a last resort
                                let username_str = username.to_string();
                                extracted_fields.insert("username".to_string(), Value::String(username_str));
                            }
                        } else {
                            // Exhaustive search for username in all fields
                            tracing::info!("Username field not found directly, searching all fields");
                            for (k, v) in fields.iter() {
                                if k.contains("username") || k.contains("name") {
                                    tracing::info!("Found potential username field '{}': {}", k, v);
                                    if v.is_string() {
                                        extracted_fields.insert("username".to_string(), v.clone());
                                        tracing::info!("Using string value from field '{}'", k);
                                        break;
                                    } else if let Some(obj) = v.as_object() {
                                        if let Some(str_val) = obj.get("string") {
                                            extracted_fields.insert("username".to_string(), str_val.clone());
                                            tracing::info!("Using string value from object in field '{}'", k);
                                            break;
                                        }
                                    }
                                }
                            }
                            
                            // If still not found, look in the entire event
                            if !extracted_fields.contains_key("username") {
                                tracing::info!("Username not found in fields, searching entire event");
                                if let Some(parsed_data) = json_value.get("parsed_json") {
                                    if let Some(username) = parsed_data.get("username") {
                                        tracing::info!("Found username in parsed_json: {}", username);
                                        extracted_fields.insert("username".to_string(), username.clone());
                                    }
                                }
                            }
                        }
                    }
                    
                    // Profile ID - CRITICAL: Must be found for username events 
                    if !extracted_fields.contains_key("profile_id") {
                        if let Some(profile_id) = fields.get("profile_id") {
                            tracing::info!("Found primary profile_id field: {}", profile_id);
                            if profile_id.is_string() {
                                extracted_fields.insert("profile_id".to_string(), profile_id.clone());
                            } else if let Some(id_str) = profile_id.as_str() {
                                extracted_fields.insert("profile_id".to_string(), Value::String(id_str.to_string()));
                            } else if let Some(obj) = profile_id.as_object() {
                                if let Some(str_val) = obj.get("string") {
                                    extracted_fields.insert("profile_id".to_string(), str_val.clone());
                                } else {
                                    // Try to convert the object to a string
                                    let id_str = serde_json::to_string(profile_id)
                                        .unwrap_or_else(|_| "".to_string());
                                    extracted_fields.insert("profile_id".to_string(), Value::String(id_str));
                                }
                            } else {
                                // Try to convert to string as a last resort
                                let id_str = profile_id.to_string();
                                extracted_fields.insert("profile_id".to_string(), Value::String(id_str));
                            }
                        } else if let Some(id_field) = fields.get("id") {
                            tracing::info!("Found id field instead of profile_id: {}", id_field);
                            extracted_fields.insert("profile_id".to_string(), id_field.clone());
                        } else {
                            // Exhaustive search for profile_id
                            tracing::info!("profile_id field not found directly, searching all fields");
                            for (k, v) in fields.iter() {
                                if k.contains("profile_id") || k == "id" || k.contains("object_id") {
                                    tracing::info!("Found potential profile_id field '{}': {}", k, v);
                                    extracted_fields.insert("profile_id".to_string(), v.clone());
                                    break;
                                }
                            }
                        }
                    }
                    
                    // Registered At - Important timestamp
                    if !extracted_fields.contains_key("registered_at") {
                        if let Some(timestamp) = fields.get("registered_at") {
                            tracing::info!("Found registered_at field: {}", timestamp);
                            if timestamp.is_number() {
                                extracted_fields.insert("registered_at".to_string(), timestamp.clone());
                            } else if let Some(n) = timestamp.as_u64() {
                                extracted_fields.insert("registered_at".to_string(), Value::Number(n.into()));
                            } else if let Some(s) = timestamp.as_str() {
                                // Try to parse string as number
                                if let Ok(n) = s.parse::<u64>() {
                                    extracted_fields.insert("registered_at".to_string(), Value::Number(n.into()));
                                } else {
                                    // Keep as string
                                    extracted_fields.insert("registered_at".to_string(), timestamp.clone());
                                }
                            } else {
                                // Use whatever we got
                                extracted_fields.insert("registered_at".to_string(), timestamp.clone());
                            }
                        } else {
                            // Try to use any timestamp field
                            for (k, v) in fields.iter() {
                                if k.contains("timestamp") || k.contains("created_at") || k.contains("time") {
                                    tracing::info!("Found potential timestamp field '{}': {}", k, v);
                                    extracted_fields.insert("registered_at".to_string(), v.clone());
                                    break;
                                }
                            }
                            
                            // If still not found, use current timestamp as fallback
                            if !extracted_fields.contains_key("registered_at") {
                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();
                                extracted_fields.insert("registered_at".to_string(), Value::Number(now.into()));
                                tracing::info!("No timestamp found, using current time: {}", now);
                            }
                        }
                    }
                    
                    // Owner address
                    if !extracted_fields.contains_key("owner_address") {
                        if let Some(owner) = fields.get("owner") {
                            tracing::info!("Found owner field: {}", owner);
                            extracted_fields.insert("owner_address".to_string(), owner.clone());
                        } else if let Some(owner) = fields.get("owner_address") {
                            tracing::info!("Found owner_address field: {}", owner);
                            extracted_fields.insert("owner_address".to_string(), owner.clone());
                        } else {
                            // Look for any field that might contain an address
                            for (k, v) in fields.iter() {
                                if k.contains("address") || k.contains("creator") || k.contains("sender") {
                                    tracing::info!("Found potential owner field '{}': {}", k, v);
                                    extracted_fields.insert("owner_address".to_string(), v.clone());
                                    break;
                                }
                            }
                            
                            // If still not found, check the event sender
                            if !extracted_fields.contains_key("owner_address") && json_value.get("sender").is_some() {
                                if let Some(sender) = json_value.get("sender") {
                                    tracing::info!("Using event sender as owner_address: {}", sender);
                                    extracted_fields.insert("owner_address".to_string(), sender.clone());
                                }
                            }
                        }
                    }
                    
                    // Log extracted fields for username events
                    tracing::info!("Extracted fields for username event:");
                    for (k, v) in &extracted_fields {
                        tracing::info!("  {}: {}", k, v);
                    }
                    
                    // Add default values for any required fields that are still missing
                    if !extracted_fields.contains_key("username") {
                        tracing::warn!("Username still not found, setting empty default");
                        extracted_fields.insert("username".to_string(), Value::String("unknown".to_string()));
                    }
                    
                    if !extracted_fields.contains_key("profile_id") {
                        tracing::warn!("profile_id still not found, setting empty default");
                        extracted_fields.insert("profile_id".to_string(), Value::String("unknown".to_string()));
                    }
                    
                    if !extracted_fields.contains_key("owner_address") {
                        tracing::warn!("owner_address still not found, setting empty default");
                        extracted_fields.insert("owner_address".to_string(), Value::String("unknown".to_string()));
                    }
                } else if !extracted_fields.contains_key("username") {
                    // Try to extract username from any field that might contain it
                    if let Some(username) = fields.get("username") {
                        tracing::info!("Found username field: {}", username);
                        extracted_fields.insert("username".to_string(), username.clone());
                    }
                }
                
                // Create a special event object with our extracted fields
                let custom_event = json!(extracted_fields);
                tracing::info!("Created custom event object: {}", custom_event);
                
                // Show field extraction outcome for debug purposes
                tracing::info!("Field extraction summary:");
                tracing::info!("  display_name: {}", extracted_fields.contains_key("display_name"));
                tracing::info!("  bio: {}", extracted_fields.contains_key("bio"));
                tracing::info!("  profile_photo: {}", extracted_fields.contains_key("profile_photo"));
                tracing::info!("  cover_photo: {}", extracted_fields.contains_key("cover_photo"));
                
                // Try to parse our custom event
                if let Ok(event) = serde_json::from_value::<T>(custom_event.clone()) {
                    tracing::info!("Successfully parsed custom event");
                    return Ok(event);
                } else {
                    tracing::warn!("Failed to parse custom event");
                }
            }
        }
    }
    
    // Try direct deserialization first
    if let Ok(result) = serde_json::from_value::<T>(json_value.clone()) {
        tracing::info!("Direct deserialization successful");
        return Ok(result);
    }
    
    // If direct parse fails, try different JSON paths that might contain our data
    let parsed_json = if let Value::Object(map) = json_value {
        // Check if we have a Move event structure
        if let Some(fields) = map.get("fields") {
            tracing::info!("Found 'fields' in event data: {}", fields);
            fields.clone()
        }
        // Check content section which might contain the fields
        else if let Some(content) = map.get("content") {
            tracing::info!("Found 'content' in event data: {}", content);
            if let Some(content_obj) = content.as_object() {
                if let Some(fields) = content_obj.get("fields") {
                    tracing::info!("Found 'fields' in content: {}", fields);
                    fields.clone()
                } else {
                    content.clone()
                }
            } else {
                content.clone()
            }
        }
        // Try common patterns
        else if let Some(value) = map.get("value") {
            tracing::info!("Found 'value' in event data: {}", value);
            value.clone()
        } else if let Some(data) = map.get("data") {
            tracing::info!("Found 'data' in event data: {}", data);
            data.clone()
        } else if let Some(parsed) = map.get("parsed_json") {
            tracing::info!("Found 'parsed_json' in event data: {}", parsed);
            parsed.clone()
        } 
        // Additional fields we're interested in
        else if let Some(bio) = map.get("bio") {
            tracing::info!("Event includes bio but not in expected structure: {}", bio);
            json_value.clone() // Still use the whole object
        } else if let Some(profile_picture) = map.get("profile_picture") {
            tracing::info!("Event includes profile_picture but not in expected structure: {}", profile_picture);
            json_value.clone() // Still use the whole object
        } else if let Some(cover_photo) = map.get("cover_photo") {
            tracing::info!("Event includes cover_photo but not in expected structure: {}", cover_photo);
            json_value.clone() // Still use the whole object
        } else {
            // Just try the whole object
            tracing::info!("No recognized structure found, using entire JSON object");
            json_value.clone()
        }
    } else {
        // Just try the raw value
        tracing::info!("JSON is not an object, using raw value");
        json_value.clone()
    };

    // Try parsing with our discovered structure
    let result = serde_json::from_value(parsed_json.clone());
    
    if result.is_err() {
        tracing::info!("Parsed JSON still failed to deserialize, error: {}", result.as_ref().err().unwrap());
        
        // Last attempt - try to manually extract fields if this is a profile event
        if let Value::Object(map) = &parsed_json {
            // Log available fields for debugging
            tracing::info!("Available fields in parsed JSON: {:?}", map.keys().collect::<Vec<_>>());
        }
    }

    result.map_err(|e| anyhow!("Failed to parse event: {}", e))
}