//! Used for internal utilities and should never be public, only a simple `use` or `mod`

use crate::{API_PREFIX, MIN_VERSION, ZinharoError};
use serde::Deserialize;

/// Version for comparing between api and this library
pub struct Version {
    /// `3` of **3**.5.2
    pub major: i32,

    /// `5` of 3.**5**.2
    pub minor: i32,

    /// `2` of 3.5.**2**
    pub patch: i32,
}

impl Version {
    /// Creates a [Version] from a string like `2.4.23`
    pub fn from_str(ver_str: &str) -> Self {
        let version_vec: Vec<i32> = ver_str
            .split(".")
            .map(|s| s.parse::<i32>().unwrap())
            .collect();

        Version {
            major: version_vec[0],
            minor: version_vec[1],
            patch: version_vec[2],
        }
    }

    /// Creates a [Version] from a &[str]
    pub fn from_resp(resp: reqwest::blocking::Response) -> Result<Self, ZinharoError> {
        #[derive(Debug, Deserialize)]
        struct MinVersion {
            min_version: String,
        }

        match resp.status().as_u16() {
            200 => {
                let resp_json = resp.json::<ApiJson<MinVersion>>()?;
                Ok(Version::from_str(&resp_json.body.min_version))
            }
            403 => Err(ZinharoError::FirewallBlock),
            e => Err(ZinharoError::UnknownStatusCode(e)),
        }
    }

    /// Check if version is within rights
    pub fn compare_versions(&self, compare: &Version) -> bool {
        if self.major >= compare.major {
            if self.minor >= compare.minor {
                if self.patch >= compare.patch {
                    return false;
                }
            }
        }

        true
    }
}

/// Connects to api's `min_version` to ensure client is not out of date. If
/// successful, should return an empty [Result::Ok]
pub fn err_min_version(client: &reqwest::blocking::Client) -> Result<(), ZinharoError> {
    let min_version_resp = client.get(&format!("{}min_version/", API_PREFIX)).send()?;
    let min_version = Version::from_resp(min_version_resp)?;

    if MIN_VERSION.compare_versions(&min_version) {
        return Err(ZinharoError::ApiVersionInadequate);
    }

    Ok(())
}

/// Generalised API response as all of them should use this baisic schema, used
/// internally
#[derive(Debug, Deserialize)]
pub struct ApiJson<T> {
    pub status: String,
    pub body: T,
}
