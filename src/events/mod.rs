mod profile_events;
mod platform_events;
mod content_events;
mod block_list_events;
mod ip_events;
mod fee_events;

pub use profile_events::*;
pub use platform_events::*;
pub use content_events::*;
pub use block_list_events::*;
pub use ip_events::*;
pub use fee_events::*;

use anyhow::{anyhow, Result};
use mys_types::{
    base_types::SuiAddress,
    event::Event as SuiEvent,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

pub const MODULE_PREFIX_PROFILE: &str = "0x2::profile::";
pub const MODULE_PREFIX_PLATFORM: &str = "0x2::social_network::platform::";
pub const MODULE_PREFIX_SOCIAL_GRAPH: &str = "0x2::social_network::social_graph::";
pub const MODULE_PREFIX_CONTENT: &str = "0x2::social_network::";
pub const MODULE_PREFIX_BLOCK_LIST: &str = "0x2::block_list::";
pub const MODULE_PREFIX_MY_IP: &str = "0x2::my_ip::";
pub const MODULE_PREFIX_PROOF_OF_CREATIVITY: &str = "0x2::proof_of_creativity::";
pub const MODULE_PREFIX_FEE_DISTRIBUTION: &str = "0x2::fee_distribution::";

/// Parse an event from a SuiEvent into a specific event type
pub fn parse_event<T: DeserializeOwned>(event: &SuiEvent) -> Result<T> {
    let json_data = serde_json::to_string(&event.parsed_json)?;
    
    // Attempt to deserialize the event data into the specific event type
    let parsed: T = serde_json::from_str(&json_data)
        .map_err(|e| anyhow!("Failed to parse event data: {}", e))?;
    
    Ok(parsed)
}

/// Helper function to extract a module name from a type string
pub fn extract_module(type_str: &str) -> &str {
    match type_str.rfind("::") {
        Some(pos) => {
            let module_with_type = &type_str[..pos];
            match module_with_type.rfind("::") {
                Some(module_pos) => &module_with_type[module_pos + 2..],
                None => module_with_type, // Unlikely case, would be malformed
            }
        }
        None => type_str, // Unlikely case, would be malformed
    }
}

/// Convert a string to a SuiAddress
pub fn string_to_address(address: &str) -> Result<SuiAddress> {
    // Remove 0x prefix if present
    let address_str = if address.starts_with("0x") {
        &address[2..]
    } else {
        address
    };
    
    // Parse address string to SuiAddress
    let address_bytes = hex::decode(address_str)
        .map_err(|e| anyhow!("Failed to decode address hex: {}", e))?;
    
    if address_bytes.len() != 32 {
        return Err(anyhow!("Invalid address length: {}", address_bytes.len()));
    }
    
    let mut addr = [0u8; 32];
    addr.copy_from_slice(&address_bytes);
    
    Ok(SuiAddress::from_bytes(addr)?)
}