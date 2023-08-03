use crate::db::DB;
use crate::handlers;
use warp::Filter;

fn with_db(db: DB) -> impl Filter<Extract = (DB,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
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

pub fn init_filters(
    db: DB,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    get_networks(db.clone())
        .or(get_rpcs(db.clone()))
        .or(get_block_explorers(db.clone()))
        .or(get_tokens(db))
}
