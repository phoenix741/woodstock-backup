use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

/// DTO for the overall health status of the storage pool.
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct PoolHealthStatusDto {
    /// Overall health indicator (false if dirty state detected)
    pub healthy: bool,
    /// Whether the pool is in a dirty state (crashed during pending pool operations)
    pub is_dirty: bool,
    /// Number of pending pool operations
    pub pending_count: i32,
}
