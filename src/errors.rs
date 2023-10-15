use thiserror::Error;
use warp::reject;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Db Error: {0}")]
    DbError(sqlx::Error),

    #[error("The game `{0}` for network `{1}` wasn't found")]
    GameDoesntExist(i64, String),

    #[error("The game with ID: `{0}` doesn't exist")]
    GameWithIDDoesntExist(i64),

    #[error("Bad signature provided address: `{0}` message: `{1}` signature: `{2}`")]
    BadSignature(String, String, String),

    #[error("{0}")]
    ArbitraryError(String),

    #[error("The auth signature is too old")]
    OldSignature,
}

impl reject::Reject for ApiError {}
