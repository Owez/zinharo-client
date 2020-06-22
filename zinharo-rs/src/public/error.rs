//! Contains error enum and trait implamentations

/// General error enums relating to the function of `zinharo-rs`
#[derive(Debug)]
pub enum ZinharoError {
    /// Given credentials where wrong when trying to login
    BadCredentials,

    /// This zinharo-rs client is too old to properly connect to the API
    ApiVersionInadequate,

    /// Encapsulates [reqwest::Error] for errors relating to API interactivity
    ReqwestError(reqwest::Error),

    /// The API returned an unknown response when interacting with it
    UnknownStatusCode(u16),

    /// When the API issues a ratelimit notice on how many requests can be sent
    Ratelimited,

    /// Encapsulates an [std::io]-based error
    IOError(std::io::Error),

    /// When no jobs are currently avalible whilst fetching
    NoJobsAvailable,

    /// The unique username given (commonly to [ZinharoAccess::signup]) has
    /// already been taken by another user
    UsernameTaken,

    /// Only used on min_version to check there is no firewall (if the simple
    /// version check gets a `403`). If you get this error returned, it means
    /// that you have been hit by cloudflare/internal firewall due to tor/ddossed
    /// website
    FirewallBlock,
}

impl From<reqwest::Error> for ZinharoError {
    fn from(error: reqwest::Error) -> Self {
        ZinharoError::ReqwestError(error)
    }
}

impl From<std::io::Error> for ZinharoError {
    fn from(error: std::io::Error) -> Self {
        ZinharoError::IOError(error)
    }
}
