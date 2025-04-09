// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// Events for platform blocking from platform.move
/// These MUST match the exact event names in the Move contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformBlockedProfileEvent {
    pub platform_id: String,
    pub profile_id: String,
    pub blocked_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformUnblockedProfileEvent {
    pub platform_id: String,
    pub profile_id: String,
    pub unblocked_by: String,
}

 