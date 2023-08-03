use crate::db::DB;
use crate::errors::ApiError;
use crate::models::json_responses::{
    BlockExplorers, InfoText, JsonResponse, Networks, ResponseBody, Rpcs, Status, Tokens,
};
use serde::Serialize;
use warp::http::StatusCode;
use warp::Reply;
use warp::{reject, reply::Response as WarpResponse};

fn get_response_status_json<T: Serialize>(status_code: StatusCode, message: T) -> impl warp::Reply {
    Ok(warp::reply::with_status(
        warp::reply::json(&message),
        status_code,
    ))
}

pub fn gen_info_response(info: &str) -> WarpResponse {
    get_response_status_json(
        StatusCode::OK,
        JsonResponse {
            status: Status::OK,
            body: ResponseBody::InfoText(InfoText {
                message: info.into(),
            }),
        },
    )
    .into_response()
}

pub fn gen_arbitrary_response(info: ResponseBody) -> WarpResponse {
    get_response_status_json(
        StatusCode::OK,
        JsonResponse {
            status: Status::OK,
            body: info,
        },
    )
    .into_response()
}

pub async fn get_networks(db: DB) -> Result<WarpResponse, warp::Rejection> {
    let networks = db
        .query_all_networks()
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Networks(Networks {
        info: networks,
    })))
}

pub async fn get_rpcs(network_id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let rpcs = db
        .query_all_rpcs(network_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Rpcs(Rpcs { rpcs })))
}

pub async fn get_block_explorers(network_id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let explorers = db
        .query_all_block_explorers(network_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::BlockExplorers(
        BlockExplorers { explorers },
    )))
}

pub async fn get_tokens(network_id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let tokens = db
        .query_all_tokens(network_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Tokens(Tokens {
        tokens,
    })))
}
