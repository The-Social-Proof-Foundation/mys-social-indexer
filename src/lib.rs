// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

pub mod api;
pub mod blockchain;
pub mod config;
pub mod db;
pub mod events;
pub mod models;
pub mod schema;

use once_cell::sync::OnceCell;

// Global package address (default value that can be overridden)
static MYSOCIAL_PACKAGE_ADDRESS: OnceCell<String> = OnceCell::new();

/// Default MySocial package address if not set via environment
pub const DEFAULT_MYSOCIAL_PACKAGE_ADDRESS: &str = "0xafb4d47301b3abd7174303822ac41e1853399738ed70fcb7a631486b152dc696";

/// Module names within the MySocial package
pub const PROFILE_MODULE_NAME: &str = "profile";
pub const PLATFORM_MODULE_NAME: &str = "platform";
pub const SOCIAL_GRAPH_MODULE_NAME: &str = "social_graph";

/// Common struct names
pub const PROFILE_STRUCT_NAME: &str = "Profile";

/// Set the MySocial package address
pub fn set_mysocial_package_address(address: String) {
    MYSOCIAL_PACKAGE_ADDRESS.set(address).unwrap_or_else(|_| {
        tracing::warn!("MySocial package address already set, ignoring new value");
    });
}

/// Get the MySocial package address
pub fn get_mysocial_package_address() -> &'static str {
    // Use hardcoded address as fallback if not set via environment variable
    MYSOCIAL_PACKAGE_ADDRESS.get().map(|s| s.as_str()).unwrap_or(DEFAULT_MYSOCIAL_PACKAGE_ADDRESS)
}

/// Get all package addresses to monitor for events
/// Currently there's only one package, but this could be expanded later if needed
pub fn get_monitored_package_addresses() -> Vec<&'static str> {
    vec![get_mysocial_package_address()]
}

/// Backward compatibility functions - these now just return the main package address

/// Get the profile package address (same as the main package)
pub fn get_profile_package_address() -> &'static str {
    get_mysocial_package_address()
}

/// Get the platform package address (same as the main package)
pub fn get_platform_package_address() -> &'static str {
    get_mysocial_package_address()
}