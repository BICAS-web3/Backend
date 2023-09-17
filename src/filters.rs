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

// NETWORKS
pub fn get_networks(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("list")
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_networks)
}

pub fn network(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("network").and(get_networks(db))
}

// RPCS
pub fn get_rpcs(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get" / i64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_rpcs)
}

pub fn rpc(db: DB) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("rpc").and(get_rpcs(db))
}

// EXPLORERS
pub fn get_all_explorers(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("list")
        .and(with_db(db))
        .and_then(handlers::get_all_explorers)
}

pub fn get_block_explorers(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get" / i64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_block_explorers)
}

pub fn block_explorer(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("block_epxlorer").and(get_block_explorers(db.clone()).or(get_all_explorers(db)))
}

// TOKENS
pub fn get_tokens(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get" / i64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_tokens)
}

pub fn token(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("token").and(get_tokens(db))
}

// GAMES
pub fn get_game(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get" / i64 / String)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_game)
}

pub fn get_game_by_id(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get" / i64)
        .and(with_db(db))
        .and_then(handlers::get_game_by_id)
}

pub fn game(db: DB) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("game").and(get_game(db.clone()).or(get_game_by_id(db)))
}

// PLAYER
pub fn get_nickname(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get" / String)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get_nickname)
}

pub fn set_nickname(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("set")
        .and(warp::post())
        .and(json_body_set_nickname())
        .and_then(with_signature)
        .and(with_db(db))
        .and_then(handlers::set_nickname)
}

pub fn get_player(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get" / String)
        .and(with_db(db))
        .and_then(handlers::get_player)
}

pub fn player(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("player").and(
        get_player(db.clone())
            .or(warp::path("nickname").and(get_nickname(db.clone()).or(set_nickname(db)))),
    )
}

// ABI
pub fn get_abi(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("get" / String)
        .and(with_db(db))
        .and_then(handlers::get_abi)
}

pub fn abi(db: DB) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("abi").and(get_abi(db))
}

// BETS
pub fn get_player_bets(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("player" / String / ..)
        .and(
            warp::path::param::<i64>()
                .map(Some)
                .or_else(|_| async { Ok::<(Option<i64>,), std::convert::Infallible>((None,)) }),
        )
        .and(warp::path::end())
        .and(with_db(db))
        .and_then(handlers::get_player_bets)
}

pub fn get_player_bets_inc(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("player" / "inc" / String / ..)
        .and(
            warp::path::param::<i64>()
                .map(Some)
                .or_else(|_| async { Ok::<(Option<i64>,), std::convert::Infallible>((None,)) }),
        )
        .and(warp::path::end())
        .and(with_db(db))
        .and_then(handlers::get_player_bets_inc)
}

pub fn get_game_bets(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("game" / i64)
        .and(with_db(db))
        .and_then(handlers::get_game_bets)
}

pub fn get_network_bets(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("network" / i64)
        .and(with_db(db))
        .and_then(handlers::get_network_bets)
}

pub fn get_all_last_bets(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("list")
        .and(with_db(db))
        .and_then(handlers::get_all_last_bets)
}

pub fn get_bets_for_game(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("game" / String)
        .and(with_db(db))
        .and_then(handlers::get_bets_for_game)
}

pub fn bets(db: DB) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("bets").and(
        get_player_bets(db.clone())
            .or(get_game_bets(db.clone()))
            .or(get_network_bets(db.clone()))
            .or(get_all_last_bets(db.clone()))
            .or(get_bets_for_game(db.clone()).or(get_player_bets_inc(db))),
    )
}

// pub fn get_full_game(
//     db: DB,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone{
//     warp::path!("get_full_game" / String)
//         .and(with_db(db))
//         .and_then(handlers::get_full_game)
// }

pub fn init_filters(
    db: DB,
    bet_sender: BetSender,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    //get_networks(db.clone())
    // .or(get_rpcs(db.clone()))
    // .or(get_block_explorers(db.clone()))
    // .or(get_tokens(db.clone()))
    // .or(get_game(db.clone()))
    // .or(get_nickname(db.clone()))
    // .or(set_nickname(db.clone()))
    // .or(get_player(db.clone()))
    // .or(get_player_bets(db.clone()))
    // .or(get_game_bets(db.clone()))
    // .or(get_network_bets(db.clone()))
    // .or(get_abi(db.clone()))
    // .or(get_all_last_bets(db.clone()))
    // .or(get_all_explorers(db.clone()))
    // .or(get_game_by_id(db.clone()))
    // .or(get_bets_for_game(db.clone()))
    network(db.clone())
        .or(rpc(db.clone()))
        .or(block_explorer(db.clone()))
        .or(token(db.clone()))
        .or(game(db.clone()))
        .or(player(db.clone()))
        .or(abi(db.clone()))
        .or(bets(db.clone()))
        .or(warp::path!("updates")
            .and(warp::ws())
            .and(with_db(db))
            .and(with_channel(bet_sender))
            .map(|ws: warp::ws::Ws, db, ch| {
                ws.on_upgrade(move |socket| handlers::websockets_handler(socket, db, ch))
            }))
}
