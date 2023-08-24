use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::communication::BetReceiver;
use crate::config;
use crate::db::DB;
use crate::errors::ApiError;
use crate::models::db_models::Nickname;
use crate::models::json_requests::{self, WebsocketsIncommingMessage};
use crate::models::json_responses::{
    Bets, BlockExplorers, InfoText, JsonResponse, Networks, ResponseBody, Rpcs, Status, Tokens,
};
use futures::stream::SplitStream;
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::time::{sleep, Duration};
use tracing::{debug, error};
use warp::http::StatusCode;
use warp::ws::{Message, WebSocket};
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
        networks,
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
        .query_block_explorers(network_id)
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
        .unwrap_or({
            debug!("Nickname for an address `{}` wasn't found", address);
            Nickname {
                id: 0,
                address: address.clone(),
                nickname: address,
            }
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
        .unwrap_or_else(|| {
            debug!("Player with address `{}` wasn't foung", address);
            Default::default()
        });

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

pub async fn get_abi(signature: String, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let abi = db
        .query_abi(&signature)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Abi(abi)))
}

pub async fn get_all_last_bets(db: DB) -> Result<WarpResponse, warp::Rejection> {
    let bets = db
        .query_all_latest_bets(*config::PAGE_SIZE)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Bets(Bets { bets })))
}

pub async fn get_all_explorers(db: DB) -> Result<WarpResponse, warp::Rejection> {
    let explorers = db
        .query_all_block_explorers()
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::BlockExplorers(
        BlockExplorers { explorers },
    )))
}

pub async fn get_game_by_id(game_id: i64, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let game = db
        .query_game_by_id(game_id)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?
        .ok_or(reject::custom(ApiError::GameWithIDDoesntExist(game_id)))?;

    Ok(gen_arbitrary_response(ResponseBody::Game(game)))
}

pub async fn get_bets_for_game(game_name: String, db: DB) -> Result<WarpResponse, warp::Rejection> {
    let bets = db
        .query_bets_for_game_name(&game_name, *config::PAGE_SIZE)
        .await
        .map_err(|e| reject::custom(ApiError::DbError(e)))?;

    Ok(gen_arbitrary_response(ResponseBody::Bets(Bets { bets })))
}

pub async fn websockets_subscriptions_reader(
    mut socket: SplitStream<WebSocket>,
    subscriptions_propagation: UnboundedSender<WebsocketsIncommingMessage>,
    _db: DB,
    run: Arc<AtomicBool>,
) {
    while run.load(Ordering::Relaxed) {
        let message = socket.next().await;
        match message {
            Some(m) => match m {
                Ok(message) => {
                    if let Ok(message) = message.to_str() {
                        let message: WebsocketsIncommingMessage =
                            match serde_json::from_str(message) {
                                Ok(m) => m,
                                Err(e) => {
                                    error!("Error parsing message `{}` | Error: {:?}", message, e);
                                    continue;
                                }
                            };

                        if let Err(e) = subscriptions_propagation.send(message) {
                            error!("Error propagating message {:?}", e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("Error on a websocket: {:?}", e);
                    break;
                }
            },
            None => {
                break;
            }
        }
    }
}

pub async fn websockets_handler(socket: WebSocket, db: DB, mut channel: BetReceiver) {
    debug!("New connection {:?}", &socket);
    let (mut ws_tx, ws_rx) = socket.split();
    let mut subscriptions: HashSet<String> = Default::default();
    let mut subscribed_all: bool = false;

    let (subscriptions_tx, mut subscriptions_rx) = unbounded_channel();

    let run = Arc::new(AtomicBool::new(true));
    tokio::spawn(websockets_subscriptions_reader(
        ws_rx,
        subscriptions_tx,
        db,
        run.clone(),
    ));

    loop {
        tokio::select! {
            bet = channel.recv() => {
                match bet{
                    Ok(bet) => {
                        if !subscribed_all && subscriptions.get(&bet.game_name).is_none(){
                            continue;
                        }

                        ws_tx
                            .send(Message::text(serde_json::to_string(&bet).unwrap()))
                            .await
                            .unwrap();
                    },
                    Err(e) => {
                        error!("Error recieving bet {:?}", e);
                        break;
                    },
                }
            }
            _ = sleep(Duration::from_millis(5000)) => {
                ws_tx
                    .send(Message::text(serde_json::to_string(&WebsocketsIncommingMessage::Ping).unwrap()))
                    .await
                    .unwrap();
            }
            msg = subscriptions_rx.recv() => {
                match msg{
                    Some(subs) => {
                        match subs{
                            WebsocketsIncommingMessage::Subscribe{payload: s} => {
                                if subscribed_all{
                                    continue;
                                }
                                let mut end = 100-subscriptions.len();
                                if end > s.len(){
                                    end = s.len();
                                }
                                for sub in &s[0..end]{
                                    subscriptions.insert(sub.clone());

                                }
                            },
                            WebsocketsIncommingMessage::Unsubscribe{payload: s} => {
                                if subscribed_all{
                                    continue;
                                }
                                for sub in s {
                                    subscriptions.remove(&sub);
                                }
                            },
                            WebsocketsIncommingMessage::SubscribeAll => {
                                subscribed_all = true;
                            },
                            WebsocketsIncommingMessage::UnsubscribeAll => {
                                subscribed_all = false;
                            },
                            WebsocketsIncommingMessage::Ping => {}
                        }
                    },
                    None => {
                        break;
                    },
                }
            }
        }
    }

    run.store(false, Ordering::Relaxed);
}
