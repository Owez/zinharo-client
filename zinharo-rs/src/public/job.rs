//! Contains a simple job representation currently. Will be updated in the future

use chrono::{DateTime, Utc};

/// A representation of a completed job, not to be confused with [ZinharoQueuedJob]
pub struct ZinharoJob {
    /// ID of job
    pub id: i32,

    /// Password found by job
    pub password: String,

    /// Client that posted job
    pub client_id: i32,

    /// Id of hash reffered to
    pub hash_id: i32,

    /// When job was created
    pub created: DateTime<Utc>,
}
