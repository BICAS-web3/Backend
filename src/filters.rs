use crate::communication::BetReceiver;
use crate::communication::BetSender;
use crate::db::DB;
use crate::errors::ApiError;
use crate::handlers;
use crate::models::json_requests;
use crate::tools;
use warp::reject;
use warp::Filter;

fn with_db(db: DB) -> impl Filter<Extract = (DB,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_channel(
    ch: BetSender,
) -> impl Filter<Extract = (BetReceiver,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || ch.subscribe())
}

async fn with_signature<'a>(
    credentials: json_requests::SetNickname,
) -> Result<json_requests::SetNickname, warp::Rejection> {
    if tools::verify_signature(
        &credentials.address,
        &credentials.nickname,
        &credentials.signature,
    ) {
        Ok(credentials)
    } else {
        Err(reject::custom(ApiError::BadSignature(
            credentials.address.to_string(),
            credentials.nickname.to_string(),
            credentials.signature,
        )))
    }
}

fn json_body_set_nickname(
) -> impl Filter<Extract = (json_requests::SetNickname,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

pub fn get_networks(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get_networks")
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_networks)
}

pub fn get_rpcs(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get_rpcs" / i64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_rpcs)
}

pub fn get_block_explorers(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get_explorers" / i64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_block_explorers)
}

pub fn get_tokens(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get_tokens" / i64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_tokens)
}

pub fn get_game(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get_game" / i64 / String)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_game)
}

pub fn get_nickname(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get_nickame" / String)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_nickname)
}

pub fn set_nickname(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("set_nickname")
        .and(json_body_set_nickname())
        .and_then(with_signature)
        .and(with_db(db))
        .and_then(handlers::set_nickname)
}

pub fn get_player(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get_player" / String)
        .and(with_db(db))
        .and_then(handlers::get_player)
}

pub fn get_player_bets(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get_player_bets" / String)
        .and(
            warp::path::param::<i64>()
                .map(Some)
                .or_else(|_| async { Ok::<(Option<i64>,), std::convert::Infallible>((None,)) }),
        )
        .and(warp::path::end())
        .and(with_db(db))
        .and_then(handlers::get_player_bets)
}

pub fn get_game_bets(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get_game_bets" / i64)
        .and(with_db(db))
        .and_then(handlers::get_game_bets)
}

pub fn get_network_bets(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get_network_bets" / i64)
        .and(with_db(db))
        .and_then(handlers::get_network_bets)
}

// pub fn websockets(bet_receiver: BetReceiver) -> i64 {
//     warp::path!("updates")
//         .and(warp::ws())
//         .and(with_channel(bet_receiver))
//         .map(|ws: warp::ws::Ws, ch| {
//             ws.on_upgrade(move |socket| handlers::websockets_handler(socket, ch))
//         })
// }

pub fn init_filters(
    db: DB,
    bet_sender: BetSender,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    get_networks(db.clone())
        .or(get_rpcs(db.clone()))
        .or(get_block_explorers(db.clone()))
        .or(get_tokens(db.clone()))
        .or(get_game(db.clone()))
        .or(get_nickname(db.clone()))
        .or(set_nickname(db.clone()))
        .or(get_player(db.clone()))
        .or(get_player_bets(db.clone()))
        .or(get_game_bets(db.clone()))
        .or(get_network_bets(db))
        .or(warp::path!("updates")
            .and(warp::ws())
            .and(with_channel(bet_sender))
            .map(|ws: warp::ws::Ws, ch| {
                ws.on_upgrade(move |socket| handlers::websockets_handler(socket, ch))
            }))
}
