use crate::models::db_models::{Bet, BetInfo, GameInfo, NetworkInfo};
use crate::{communication::*, db::DB};
use chrono::Utc;
use ethabi::ethereum_types::H256;
use ethabi::{ParamType, Token};
use futures::StreamExt;
use sqlx::types::BigDecimal;
use std::collections::HashMap;

use std::str::FromStr;
use std::time;
use tracing::{debug, error, warn};

use web3::types::{FilterBuilder, H160};

type GameInnerInfo = HashMap<H256, (H160, (Vec<ParamType>, Vec<String>), GameInfo)>;

pub async fn start_network_handlers(db: DB, bet_sender: BetSender) {
    // channels
    let (db_sender, db_receiver) = unbounded_channel();

    // spawn db listener
    // TODO: make a proper db listener
    tokio::spawn(db_listener(db_receiver, db.clone()));

    let networks = db.query_all_networks().await.unwrap();
    for network in networks {
        debug!(
            "Staring games handlers on network: `{:?}`",
            network.network_id
        );
        let rpcs = db
            .query_all_rpcs(network.network_id)
            .await
            .unwrap()
            .into_iter()
            .map(|rpc| rpc.url)
            .collect();
        let games: GameInnerInfo = db
            .query_all_games_infos(network.network_id)
            .await
            .unwrap()
            .into_iter()
            .map(|game| {
                debug!("Producing data for game `{:?}`", game.id);
                let mut game_address: [u8; 20] = [0; 20];
                hex::decode_to_slice(game.address[2..].as_bytes(), &mut game_address).unwrap();

                let mut event_signature: [u8; 32] = [0; 32];
                hex::decode_to_slice(game.event_signature[2..].as_bytes(), &mut event_signature)
                    .unwrap();
                (
                    H256::from_slice(&event_signature),
                    (
                        H160::from_slice(&game_address),
                        (
                            serde_json::from_str::<Vec<ParamType>>(&game.event_types).unwrap(),
                            game.event_names
                                .split(' ')
                                .map(|s| s.to_string())
                                .collect::<Vec<String>>(),
                        ),
                        game,
                    ),
                )
            })
            .collect();
        tokio::spawn(network_handler(
            network,
            rpcs,
            games,
            db_sender.clone(),
            bet_sender.clone(),
            db.clone(),
        ));
    }
}

#[allow(unused_assignments)]
pub async fn network_handler(
    network: NetworkInfo,
    rpc_urls: Vec<String>,
    games: GameInnerInfo,
    db_sender: DbSender,
    bet_sender: BetSender,
    db: DB,
) {
    let mut restart = true;

    while restart {
        restart = false;
        let transport = rpc_urls
            .iter()
            .find_map(|url| web3::transports::Http::new(url).ok())
            .unwrap();

        debug!("Starting listening to rpc: {:?}", transport);

        let web3 = web3::Web3::new(transport);

        let filter = FilterBuilder::default()
            .address(games.iter().map(|item| item.1 .0).collect())
            .build();

        let filter = web3.eth_filter().create_logs_filter(filter).await.unwrap();

        let logs_stream = filter.stream(time::Duration::from_secs(1));
        futures::pin_mut!(logs_stream);

        loop {
            let log = match logs_stream.next().await {
                Some(Ok(log)) => log,
                Some(Err(e)) => {
                    error!(
                        "Error reading log stream for Network: `{:?}` {:?}",
                        network.network_id, e
                    );
                    restart = true;
                    break;
                }
                None => {
                    warn!(
                        "Connection for Network `{:?}` is closed",
                        network.network_id
                    );
                    restart = true;
                    break;
                }
            };
            debug!("Log received {:?}", log);

            let topics = log.topics;
            let (_, (types, names), game) = match games.get(&topics[0]) {
                Some(r) => r,
                None => {
                    warn!("No event signature `{:?}` was found", topics[0]);
                    continue;
                }
            };

            let decoded_data = match ethabi::decode(types, &log.data.0) {
                Ok(data) => data,
                Err(e) => {
                    error!(
                        "Network: `{:?}` error on decoding data: {:?}",
                        network.network_id, e
                    );
                    continue;
                }
            };
            debug!("Decoded data {:?}", &decoded_data);
            let decoded_data: HashMap<String, Token> = names
                .iter()
                .cloned()
                .zip(decoded_data.into_iter())
                .collect();
            debug!("Decoded data as hashmap {:?}", &decoded_data);

            let bet = Bet {
                id: 0,
                transaction_hash: format!("0x{}", hex::encode(log.transaction_hash.unwrap().0)),
                player: format!("0x{}", hex::encode(&topics[1].0[12..])),
                timestamp: Utc::now(),
                game_id: game.id,
                wager: BigDecimal::from_str(
                    &decoded_data
                        .get("wager")
                        .unwrap()
                        .clone()
                        .into_uint()
                        .unwrap()
                        .to_string(),
                )
                .unwrap(),
                token_address: format!(
                    "0x{}",
                    hex::encode(
                        decoded_data
                            .get("tokenAddress")
                            .unwrap()
                            .clone()
                            .into_address()
                            .unwrap()
                            .0
                    )
                ),
                network_id: game.network_id,
                bets: match match decoded_data.get("numGames") {
                    Some(t) => t,
                    None => {
                        error!("Could not find token `numGames`");
                        continue;
                    }
                }
                .clone()
                .into_uint()
                {
                    Some(n) => n,
                    None => {
                        error!("Could not parse uint for `numGames`");
                        continue;
                    }
                }
                .as_u32() as i64,
                multiplier: 1.0,
                profit: BigDecimal::from_str(
                    &decoded_data
                        .get("payout")
                        .unwrap()
                        .clone()
                        .into_uint()
                        .unwrap()
                        .to_string(),
                )
                .unwrap(),
            };

            if let Ok(token) = db.query_token(&bet.token_address).await {
                let bet_info = BetInfo {
                    id: 0,
                    transaction_hash: bet.transaction_hash.clone(),
                    player: bet.player.clone(),
                    player_nickname: db
                        .query_nickname(&bet.player)
                        .await
                        .unwrap_or(None)
                        .map(|player| player.nickname),
                    timestamp: bet.timestamp,
                    game_id: bet.game_id,
                    game_name: game.name.clone(),
                    wager: bet.wager.clone(),
                    token_address: bet.token_address.clone(),
                    token_name: token.name,
                    network_id: bet.network_id,
                    network_name: network.network_name.clone(),
                    bets: bet.bets,
                    multiplier: bet.multiplier,
                    profit: bet.profit.clone(),
                };

                if let Err(e) = db_sender.send(bet.clone()) {
                    error!("Error sending bet to db {:?}", e);
                    continue;
                }

                if let Err(e) = bet_sender.send(bet_info) {
                    error!("Error propagating bet {:?}", e);
                    continue;
                }
            } else {
                error!("Token `{}` not found", &bet.token_address);
            }
        }
    }
}

pub async fn db_listener(mut receiver: DbReceiver, db: DB) {
    while let Some(msg) = receiver.recv().await {
        if let Err(e) = db.place_bet(&msg).await {
            error!("Error placing bet {:?}", e);
        }
    }
}
