// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

pub mod profile;
pub mod indexer;
pub mod social_graph;
pub mod platform;
pub mod blocking;
pub mod profile_events;

pub use profile::*;
pub use indexer::*;
pub use social_graph::*;

// Export platform models except for the duplicated event types
pub use platform::{
    Platform, NewPlatform, UpdatePlatform,
    PlatformModerator, NewPlatformModerator,
    PlatformBlockedProfile, NewPlatformBlockedProfile,
    PlatformEvent, NewPlatformEvent,
    PlatformWithDetails, PlatformCreatedEvent, PlatformApprovalChangedEvent,
    PlatformUpdatedEvent, PlatformStatus, ModeratorAddedEvent, ModeratorRemovedEvent,
    UserJoinedPlatformEvent, UserLeftPlatformEvent,
    NewPlatformMembership,
    PLATFORM_STATUS_DEVELOPMENT, PLATFORM_STATUS_ALPHA, PLATFORM_STATUS_BETA,
    PLATFORM_STATUS_LIVE, PLATFORM_STATUS_MAINTENANCE, PLATFORM_STATUS_SUNSET, PLATFORM_STATUS_SHUTDOWN
};

// Export blocking models - these event types should be used instead of the ones in platform module
pub use blocking::*;

// Export profile events models
pub use profile_events::*;