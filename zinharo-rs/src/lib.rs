//! `zinharo-rs` is a simple API binding for use in backend servers and to even
//! run on users computers for pentesting efforts. The main structure used is
//! [ZinharoQueuedJob] with [crate::launch] being used for authentication for many
//! routes. The usage and main goal of this library is suppost to be as
//! developer-friendly as possible.

#[cfg(test)]
extern crate rand;

mod public;
mod utils; // ensure this is never publically exposed

pub use public::*;
use utils::Version; // only for internal versioning

/// The prefix to use when connecting to client. Ususally `127.0.0.1:5000/api/`
/// for debug.
pub const API_PREFIX: &str = "http://0.0.0.0:8082/";

/// Minimum API version allowed
pub const MIN_VERSION: Version = Version {
    major: 0,
    minor: 0,
    patch: 1,
};

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    /// Debug admin username
    const USERNAME: &str = "coolman";

    /// Debug admin password
    const PASSWORD: &str = "englandismycity";

    /// Attempts to login using the debug admin credentials
    #[test]
    fn admin_login() {
        ZinharoAccess::login(USERNAME, PASSWORD).unwrap();
    }

    /// Uses [ZinharoQueuedJob] to fetch a job
    #[test]
    fn fetch_job() {
        let access = ZinharoAccess::login(USERNAME, PASSWORD).unwrap();
        ZinharoQueuedJob::new(&access).unwrap();
    }

    /// Fetches a job then submits it with dummy password
    #[test]
    fn submit_job() {
        let access = ZinharoAccess::login(USERNAME, PASSWORD).unwrap();
        let job = ZinharoQueuedJob::new(&access).unwrap();

        job.submit(&access, "dummypassword").unwrap();
    }

    /// Gets a job then submits a report on it
    #[test]
    fn report_job() {
        let access = ZinharoAccess::login(USERNAME, PASSWORD).unwrap();
        let job = ZinharoQueuedJob::new(&access).unwrap();

        job.report(
            &access,
            Some("This is some infomation on why this job was reported"),
        )
        .unwrap();
    }

    /// Attempts to sign into the api
    #[test]
    fn signup() {
        let username: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();
        let password: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();

        ZinharoAccess::signup(&username, &password).unwrap();
    }

    /// Attempts to add a `.cap` stream (dummy stream in this case)
    #[test]
    fn add_cap() {
        let access = ZinharoAccess::login(USERNAME, PASSWORD).unwrap();
        let my_cap: Vec<u8> = vec![4, 5, 43, 75, 134];

        let hash = ZinharoHash::from_cap(&access, Vec::clone(&my_cap)).unwrap();

        assert_eq!(my_cap, hash.cap); // should be same after upload
    }
}
