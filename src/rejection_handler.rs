use warp::{hyper::StatusCode, reject};

use crate::errors::ApiError;
use tracing::error;

pub async fn handle_rejection(
    err: reject::Rejection,
) -> std::result::Result<impl warp::Reply, std::convert::Infallible> {
    let (code, message) = if err.is_not_found() {
        error!("Not Found: {:?}", err);
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if let Some(e) = err.find::<ApiError>() {
        error!("Error: {:?}", e);
        match e {
            ApiError::DbError(_) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            _ => (StatusCode::BAD_REQUEST, e.to_string()),
        }
    } else {
        error!("Unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    let json = warp::reply::json(&crate::models::json_responses::JsonResponse {
        status: crate::models::json_responses::Status::Err,
        body: crate::models::json_responses::ResponseBody::ErrorText(
            crate::models::json_responses::ErrorText { error: message },
        ),
    });
    Ok(warp::reply::with_status(json, code))
}
