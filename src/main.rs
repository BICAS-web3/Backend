use std::io;

use crate::communication::*;
use config::DatabaseSettings;
use db::DB;
use rejection_handler::handle_rejection;
use std::env;
use tokio::signal;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt, EnvFilter};
use warp::Filter;

mod communication;
mod config;
mod db;
mod errors;
mod filters;
mod handlers;
mod models;
mod network_handler;
mod rejection_handler;
mod tools;

#[tokio::main]
async fn main() {
    // load .env file
    dotenvy::dotenv()
        .map_err(|e| {
            error!(error = e.to_string(), "Error loading .env");
            e
        })
        .unwrap();

    // load log config
    let env_filter = EnvFilter::from_default_env()
        .add_directive("backend=debug".parse().unwrap())
        .add_directive("hyper=warn".parse().unwrap());
    let collector = tracing_subscriber::registry().with(env_filter).with(
        fmt::Layer::new()
            .with_writer(io::stdout)
            .with_thread_names(true),
    );
    let file_appender = tracing_appender::rolling::minutely("logs", "backend_log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let collector = collector.with(
        fmt::Layer::new()
            .with_writer(non_blocking)
            .with_thread_names(true),
    );
    tracing::subscriber::set_global_default(collector).unwrap();

    info!("Starting rest api");

    let db_settings = DatabaseSettings {
        username: env::var("DB_USERNAME").unwrap(),
        password: env::var("DB_PASSWORD").unwrap(),
        host: env::var("DB_HOST").unwrap(),
        port: env::var("DB_PORT").unwrap().parse().unwrap(),
        database_name: env::var("DB_NAME").unwrap(),
    };

    debug!("Connecting to DB with settings {:?}", db_settings);

    let db = DB::new(&db_settings).await;

    info!(
        "The rest api is starting on the {:?}:{:?}",
        *config::SERVER_HOST,
        *config::SERVER_PORT
    );

    let (bet_sender, _bet_receiver) = channel(10000);

    info!("Staring networks handlers");
    network_handler::start_network_handlers(db.clone(), bet_sender.clone()).await;

    info!("Server started, waiting for CTRL+C");
    tokio::select! {
        _ = warp::serve(
            filters::init_filters(db, bet_sender).recover(handle_rejection), //.with(cors),
        )
        .run((*config::SERVER_HOST, *config::SERVER_PORT)) => {},
        _ = signal::ctrl_c() => {
            warn!("CTRL+C received, stopping process...")
        }
    }
}
