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

// Global profile package address (default value that can be overridden)
static PROFILE_PACKAGE_ADDRESS: OnceCell<String> = OnceCell::new();

/// Profile module name
pub const PROFILE_MODULE_NAME: &str = "profile";

/// Profile struct name
pub const PROFILE_STRUCT_NAME: &str = "Profile";

/// Set the profile package address
pub fn set_profile_package_address(address: String) {
    PROFILE_PACKAGE_ADDRESS.set(address).unwrap_or_else(|_| {
        tracing::warn!("Profile package address already set, ignoring new value");
    });
}

/// Get the profile package address
pub fn get_profile_package_address() -> &'static str {
    // Use hardcoded address as fallback if not set via environment variable
    // This is the address of the profile contract on the MySocial network
    PROFILE_PACKAGE_ADDRESS.get().map(|s| s.as_str()).unwrap_or(
        // MySocial profile contract address
        // Important: This should be updated with the actual deployed contract address
        "0xb943e36e17975171a69c4b4fd5f51e12273c7e9947ef9bb73afd63b3d8dc0860"
    )
}