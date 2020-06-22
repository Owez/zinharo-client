//! Contains [ZinharoHash] and implamentations of it. Used for uploading a `.cap`
//! stream and getting infomation on said `.cap`s

use crate::utils::ApiJson;
use crate::{ZinharoAccess, ZinharoError, ZinharoJob, ZinharoReport, API_PREFIX};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;

/// A representation of a hash/`.cap` file in zinharo, used mainly for applications
/// uploading hashes and getting info back (like a desktop application gui)
pub struct ZinharoHash {
    /// ID of hash
    pub id: i32,

    /// The full byte-stream of the `.cap` file
    pub cap: Vec<u8>,

    /// Jobs associated with hash
    pub jobs: Vec<ZinharoJob>,

    /// Reports associated with hash
    pub reports: Vec<ZinharoReport>,

    /// Date of hash creation
    pub created: DateTime<Utc>,
}

impl ZinharoHash {
    /// Creates a new ZinharoHash from a `.cap` vec stream
    pub fn from_cap(access: &ZinharoAccess, cap: Vec<u8>) -> Result<Self, ZinharoError> {
        let mut payload = HashMap::new();
        payload.insert("cap", base64::encode(&cap));

        let resp = access
            .client
            .post(&format!("{}hash/", API_PREFIX))
            .json(&payload)
            .send()?;

        match resp.status().as_u16() {
            200 => {
                /// Partial representation of [ZinharoJob]
                #[derive(Debug, Deserialize)]
                struct JsonJob {
                    id: i32,
                    password: String,
                    client_id: i32,
                    created: String,
                }

                /// Partial representation of [ZinharoReport]
                #[derive(Debug, Deserialize)]
                struct JsonReport {
                    id: i32,
                    info: Option<String>,
                    client_id: i32,
                    created: String,
                }

                /// Representation of large json response
                #[derive(Debug, Deserialize)]
                struct JsonHash {
                    id: i32,
                    cap: String,
                    created: String,
                    jobs: Vec<JsonJob>,
                    reports: Vec<JsonReport>,
                }

                /// Wrapper for internal [JsonHash] to fit with api schema
                #[derive(Debug, Deserialize)]
                struct JsonHashWrapper {
                    hash: JsonHash,
                }

                let resp_json = resp.json::<ApiJson<JsonHashWrapper>>()?;

                let final_created = DateTime::parse_from_rfc3339(&resp_json.body.hash.created)
                    .unwrap()
                    .with_timezone(&Utc);

                let mut final_jobs: Vec<ZinharoJob> = Vec::new();

                for job in resp_json.body.hash.jobs {
                    let job_created = DateTime::parse_from_rfc3339(&job.created)
                        .unwrap()
                        .with_timezone(&Utc);

                    let hash_id = resp_json.body.hash.id;

                    final_jobs.push(ZinharoJob {
                        id: job.id,
                        password: job.password,
                        client_id: job.client_id,
                        hash_id: hash_id,
                        created: job_created,
                    })
                }

                let mut final_reports: Vec<ZinharoReport> = Vec::new();

                for report in resp_json.body.hash.reports {
                    let report_created = DateTime::parse_from_rfc3339(&report.created)
                        .unwrap()
                        .with_timezone(&Utc);

                    let hash_id = resp_json.body.hash.id;

                    final_reports.push(ZinharoReport {
                        id: report.id,
                        info: report.info,
                        client_id: report.client_id,
                        hash_id: hash_id,
                        created: report_created,
                    })
                }

                Ok(ZinharoHash {
                    id: resp_json.body.hash.id,
                    cap: cap,
                    jobs: final_jobs,
                    reports: final_reports,
                    created: final_created,
                })
            }
            429 => Err(ZinharoError::Ratelimited),
            e => Err(ZinharoError::UnknownStatusCode(e)),
        }
    }

    // /// Updates infomation like [ZinharoHash::jobs] and [ZinharoHash::reports]
    // pub fn update_info(&self, access: &ZinharoAccess) -> Result<(), ZinharoError> {
    //     let params = [("id", self.id)];
    //     let resp = access
    //         .client
    //         .get(&format!("{}hash/", API_PREFIX))
    //         .query(&params)
    //         .send()?;
    // }
}
