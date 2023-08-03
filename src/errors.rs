use thiserror::Error;
use warp::reject;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Db Error: {0}")]
    DbError(sqlx::Error),
}

impl reject::Reject for ApiError {}
