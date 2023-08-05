use crate::config;
use crate::db::DB;
use crate::errors::ApiError;
use crate::models::db_models::Nickname;
use crate::models::json_requests;
use crate::models::json_responses::{
    Bets, BlockExplorers, InfoText, JsonResponse, Networks, ResponseBody, Rpcs, Status, Tokens,
};
use serde::Serialize;
use tracing::debug;
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

pub async fn get_game(
    network_id: i64,
    game_name: String,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let game = db
        .query_game(network_id, &game_name)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?
        .ok_or(reject::custom(ApiError::GameDoesntExist(
            network_id, game_name,
        )))?;

    Ok(gen_arbitrary_response(ResponseBody::Game(game)))
}

pub async fn get_nickname(address: String, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let nickname = db
        .query_nickname(&address)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?
        .map(|n| {
            debug!("Nickname for an address `{}` wasn't found", address);
            n
        })
        .unwrap_or(Nickname {
            id: 0,
            address: address.clone(),
            nickname: address,
        });

    Ok(gen_arbitrary_response(ResponseBody::Nickname(nickname)))
}

pub async fn set_nickname(
    credentials: json_requests::SetNickname,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    db.set_nickname(&credentials.address, &credentials.nickname)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_info_response("The nickname has been changed"))
}

pub async fn get_player(address: String, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let player = db
        .query_player(&address)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?
        .map(|p| {
            debug!("Player with address `{}` wasn't foung", address);
            p
        })
        .unwrap_or(Default::default());

    Ok(gen_arbitrary_response(ResponseBody::Player(player)))
}

pub async fn get_player_bets(
    address: String,
    last_id: Option<i64>,
    db: DB,
) -> Result<WarpResponse, warp::Rejection> {
    let bets = db
        .query_bets_for_address(&address, last_id, *config::PAGE_SIZE)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Bets(Bets { bets })))
}

pub async fn get_game_bets(game_id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let bets = db
        .query_bets_for_game(game_id, *config::PAGE_SIZE)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Bets(Bets { bets })))
}

pub async fn get_network_bets(netowork_id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let bets = db
        .query_bets_for_network(netowork_id, *config::PAGE_SIZE)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Bets(Bets { bets })))
}
