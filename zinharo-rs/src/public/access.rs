//! Contains basic auth structure that allows rest of library to function under
//! authentication

use crate::utils::{err_min_version, ApiJson};
use crate::{ZinharoError, API_PREFIX};
use serde::{Deserialize, Serialize};

/// Stores access infomation essential to many functions, used as it is more
/// convinient than a tuple
pub struct ZinharoAccess {
    /// JWT token
    pub token: String,

    /// Reqwest client
    pub client: reqwest::blocking::Client,
}

impl ZinharoAccess {
    /// Attempts to log into api and returns the reqwest client and the API token.
    /// May provide [ZinharoError::BadCredentials] or [ZinharoError::ApiVersionInadequate]
    pub fn login(username: &str, password: &str) -> Result<Self, ZinharoError> {
        let client = reqwest::blocking::Client::new();

        err_min_version(&client)?;

        let params = [("username", username), ("password", password)];
        let login_resp = client
            .get(&format!("{}auth/", API_PREFIX))
            .query(&params)
            .send()?;

        match login_resp.status().as_u16() {
            200 => {
                #[derive(Debug, Deserialize)]
                struct Token {
                    token: String,
                }

                let token = login_resp.json::<ApiJson<Token>>()?.body.token;

                Ok(ZinharoAccess {
                    token: token,
                    client: client,
                })
            }
            403 => Err(ZinharoError::BadCredentials),
            429 => Err(ZinharoError::Ratelimited),
            e => Err(ZinharoError::UnknownStatusCode(e)),
        }
    }

    /// Similar to the login function, creates a new [ZinharoAccess] but creates
    /// a new account. Beware when using this method as it is heavily ratelimited
    /// to prevent spam
    pub fn signup(username: &str, password: &str) -> Result<Self, ZinharoError> {
        let client = reqwest::blocking::Client::new();

        err_min_version(&client)?;

        /// Internal structure for sending signup params
        #[derive(Debug, Serialize)]
        struct SignupJson<'a> {
            username: &'a str,
            password: &'a str,
        }

        let signup_json = SignupJson { username, password };

        let signup_resp = client
            .post(&format!("{}auth/", API_PREFIX))
            .json(&signup_json)
            .send()?;

        match signup_resp.status().as_u16() {
            200 => {
                #[derive(Debug, Deserialize)]
                struct Token {
                    token: String,
                }

                let token = signup_resp.json::<ApiJson<Token>>()?.body.token;

                Ok(ZinharoAccess {
                    token: token,
                    client: client,
                })
            }
            403 => Err(ZinharoError::UsernameTaken),
            429 => Err(ZinharoError::Ratelimited),
            e => Err(ZinharoError::UnknownStatusCode(e)),
        }
    }
}
