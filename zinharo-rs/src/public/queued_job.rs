//! Contains a rich [ZinharoQueuedJob] queued job representation and
//! implamentations around it

use crate::utils::ApiJson;
use crate::{ZinharoAccess, ZinharoError, API_PREFIX};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

/// A single job for Zinharo, used for job management inside of server clients
pub struct ZinharoQueuedJob {
    /// Base64 string (NOTE: may get replaced by `&[u8]`)
    pub cap: Vec<u8>,

    /// Job ID
    pub id: i32,

    /// UTC creation date
    pub created: DateTime<Utc>,
}

impl ZinharoQueuedJob {
    /// Fetches a job and creates a handy new [ZinharoQueuedJob] to use
    pub fn new(access: &ZinharoAccess) -> Result<Self, ZinharoError> {
        let resp = access
            .client
            .get(&format!("{}job/", API_PREFIX))
            .bearer_auth(String::clone(&access.token))
            .send()?;

        match resp.status().as_u16() {
            200 => {
                /// Holder for [JsonJobQueued]
                #[derive(Debug, Deserialize)]
                struct JsonJob {
                    queued: JsonJobQueued,
                }

                /// Similar to [ZinharoQueuedJob] but without some parts converted
                #[derive(Debug, Deserialize)]
                struct JsonJobQueued {
                    cap: String,
                    id: i32,
                    created: String,
                }

                let resp_json = resp.json::<ApiJson<JsonJob>>()?;

                let final_cap = base64::decode(resp_json.body.queued.cap).unwrap();
                let final_created = DateTime::parse_from_rfc3339(&resp_json.body.queued.created)
                    .unwrap()
                    .with_timezone(&Utc);

                Ok(ZinharoQueuedJob {
                    cap: final_cap,
                    id: resp_json.body.queued.id,
                    created: final_created,
                })
            }
            429 => Err(ZinharoError::Ratelimited),
            404 => Err(ZinharoError::NoJobsAvailable),
            e => Err(ZinharoError::UnknownStatusCode(e)),
        }
    }

    /// Submits job when finished
    pub fn submit(&self, access: &ZinharoAccess, password: &str) -> Result<(), ZinharoError> {
        /// Temp payload used to send job info
        #[derive(Debug, Serialize)]
        struct JsonPayload<'a> {
            id: i32,
            password: &'a str,
        }

        let payload = JsonPayload {
            id: self.id,
            password: password,
        };

        let resp = access
            .client
            .post(&format!("{}job/", API_PREFIX))
            .json(&payload)
            .bearer_auth(String::clone(&access.token))
            .send()?;

        match resp.status().as_u16() {
            200 => Ok(()),
            429 => Err(ZinharoError::Ratelimited),
            e => Err(ZinharoError::UnknownStatusCode(e)),
        }
    }

    /// Reports current job with optional provided infomation
    pub fn report(&self, access: &ZinharoAccess, info: Option<&str>) -> Result<(), ZinharoError> {
        /// Temp payload used to send report info
        #[derive(Debug, Serialize)]
        struct JsonPayload<'a> {
            hash_id: i32,
            info: Option<&'a str>,
        }

        let payload = JsonPayload {
            hash_id: self.id,
            info: info,
        };

        let resp = access
            .client
            .post(&format!("{}report/", API_PREFIX))
            .json(&payload)
            .bearer_auth(String::clone(&access.token))
            .send()?;

        match resp.status().as_u16() {
            200 => Ok(()),
            429 => Err(ZinharoError::Ratelimited),
            e => Err(ZinharoError::UnknownStatusCode(e)),
        }
    }

    /// Dumps [ZinharoQueuedJob::cap] to a given filepath. This often returns
    /// [ZinharoError::IOError(_)].
    pub fn dump_cap(&self, path: PathBuf) -> Result<(), ZinharoError> {
        let mut file = File::create(path)?;
        file.write_all(&self.cap)?;

        Ok(())
    }
}
