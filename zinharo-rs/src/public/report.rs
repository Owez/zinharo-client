//! Contains a rich report representation, similar to [crate::ZinharoJob]. Will
//! probably be updated in the future

use chrono::{DateTime, Utc};

/// A representation of a report. If you are looking for how to make reports,
/// please view [ZinharoAccess::report] as [ZinharoAccess] houses cracking/server
/// related methods.
pub struct ZinharoReport {
    /// ID of report
    pub id: i32,

    /// Optional infomation given. If this is [Option::None], it may be a false
    /// report as official client always states reason
    pub info: Option<String>,

    /// ID of client that made report
    pub client_id: i32,

    /// ID of hash that the report is made for
    pub hash_id: i32,

    /// When report was filed/created
    pub created: DateTime<Utc>,
}
