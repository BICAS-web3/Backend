use crate::models::db_models::{GameInfo, NetworkInfo, Token, TokenPrice};
use crate::models::json_responses::{BetInfoResponse, Card};
use crate::{communication::*, db::DB};
use chrono::Utc;
use ethabi::ethereum_types::{H256, U256};
use ethabi::{ParamType, Token as EthToken};
use futures::StreamExt;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sqlx::types::BigDecimal;
use std::collections::HashMap;
use std::fs::File;
use tokio::time::{sleep, Duration};
use web3::contract::Contract;

use std::str::FromStr;
use std::time;
use tracing::{debug, error, warn};

use web3::types::{FilterBuilder, Log, H160};

type GameInnerInfo = HashMap<H256, (H160, (Vec<ParamType>, Vec<String>), GameInfo)>;

pub async fn start_network_handlers(db: DB, bet_sender: BetSender) {
    // channels
    let (db_sender, db_receiver) = unbounded_channel();

    // spawn db listener
    // TODO: make a proper db listener
    tokio::spawn(db_listener(db_receiver, db.clone()));

    let networks = db.query_all_networks().await.unwrap();
    for network in networks.iter() {
        debug!(
            "Staring games handlers on network: `{:?}`",
            network.network_id
        );
        let rpcs: Vec<String> = db
            .query_all_rpcs(network.network_id)
            .await
            .unwrap()
            .into_iter()
            .map(|rpc| rpc.url)
            .collect();
        if rpcs.is_empty() {
            continue;
        }
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

        let last_block = db
            .query_last_block(network.network_id)
            .await
            .unwrap()
            .map(|block| block.id as u64);

        tokio::spawn(network_handler(
            network.clone(),
            rpcs,
            games,
            db_sender.clone(),
            bet_sender.clone(),
            last_block,
        ));
    }

    let tokens = db
        .get_unique_tokens(56)
        .await
        .map_err(|e| {
            error!("Error getting list of tokens {:?}", e);
            e
        })
        .unwrap();

    tokio::spawn(token_price_handler(
        networks[0].clone(),
        vec!["https://bsc-dataseed1.binance.org".into()],
        tokens,
        db_sender,
        "0x55d398326f99059fF775485246999027B3197955".into(),
        "0x10ED43C718714eb63d5aA57B78B54704E256024E".into(),
    ));
}
async fn handle_game_log(
    log: Log,
    network: &NetworkInfo,
    games: &GameInnerInfo,
    db_sender: &DbSender,
    bet_sender: &BetSender,
) {
    debug!("Log received {:?}", log);

    let topics = log.topics;
    let (_, (types, names), game) = match games.get(&topics[0]) {
        Some(r) => r,
        None => {
            warn!("No event signature `{:?}` was found", topics[0]);
            return;
        }
    };

    let decoded_data = match ethabi::decode(types, &log.data.0) {
        Ok(data) => data,
        Err(e) => {
            error!(
                "Network: `{:?}` error on decoding data: {:?}",
                network.network_id, e
            );
            return;
        }
    };
    debug!("Decoded data {:?}", &decoded_data);
    let decoded_data: HashMap<String, EthToken> = names
        .iter()
        .cloned()
        .zip(decoded_data.into_iter())
        .collect();
    debug!("Decoded data as hashmap {:?}", &decoded_data);

    let player_hand: Option<Vec<u8>> = decoded_data.get("playerHand").cloned().map(|token| {
        token
            .into_fixed_array()
            .unwrap()
            .iter()
            .map(|token| token.clone().into_uint().unwrap().as_u32() as u8)
            .collect()
    });

    let player_hand: Option<Vec<Card>> = player_hand.map(|player_hand| {
        player_hand
            .chunks(2)
            .map(|chunk| Card {
                number: chunk[0],
                suit: chunk[1],
            })
            .collect()
    });

    let is_end_transaction = decoded_data.get("payout").is_some();

    let bet = BetInfoResponse {
        id: 0,
        transaction_hash: format!("0x{}", hex::encode(log.transaction_hash.unwrap().0)),
        player: format!("0x{}", hex::encode(&topics[1].0[12..])),
        timestamp: Utc::now(),
        game_id: game.id,
        wager: match decoded_data.get("wager") {
            Some(n) => BigDecimal::from_str(&n.clone().into_uint().unwrap().to_string()).unwrap(),
            None => BigDecimal::default(),
        },
        token_address: match decoded_data.get("tokenAddress") {
            Some(token_address) => format!(
                "0x{}",
                hex::encode(token_address.clone().into_address().unwrap().0)
            ),
            None => "".to_string(),
        },
        network_id: game.network_id,
        bets: match decoded_data.get("numGames") {
            Some(t) => match t.clone().into_uint() {
                Some(n) => n.as_u32() as i64,
                None => {
                    error!("Could not parse uint for `numGames`");
                    return;
                }
            },
            None => {
                warn!("Could not find token `numGames`");
                1
                //return;
            }
        },
        multiplier: 1.0,
        profit: match decoded_data.get("payout") {
            Some(n) => BigDecimal::from_str(&n.clone().into_uint().unwrap().to_string()).unwrap(),
            None => BigDecimal::default(),
        },

        player_nickname: Default::default(),
        game_name: Default::default(),
        token_name: Default::default(),
        network_name: Default::default(),
        player_hand,
    };

    if is_end_transaction {
        if let Err(e) = db_sender.send(DbMessage::PlaceBet(bet.clone().into())) {
            error!("Error sending bet to db {:?}", e);
            return;
        }
    }

    if let Err(e) = bet_sender.send(PropagatedBet {
        bet,
        game_name: game.name.clone(),
        network_name: network.network_name.clone(),
    }) {
        error!("Error propagating bet {:?}", e);
    }
}

#[allow(unused_assignments)]
pub async fn token_price_handler(
    _network: NetworkInfo,
    rpc_urls: Vec<String>,
    tokens: Vec<Token>,
    db_sender: DbSender,
    usdt_token_address: String,
    router_address: String,
) {
    loop {
        let abi_file = File::open("./abis/pancake.json").unwrap();
        let mut router_address_wrapped: [u8; 20] = [0; 20];
        hex::decode_to_slice(router_address[2..].as_bytes(), &mut router_address_wrapped).unwrap();

        let transport = rpc_urls
            .iter()
            .find_map(|url| web3::transports::Http::new(url).ok())
            .unwrap();

        debug!("Starting listening to rpc: {:?}", transport);

        let web3 = web3::Web3::new(transport);

        let contract = Contract::new(
            web3.eth(),
            H160::from_slice(&router_address_wrapped),
            ethabi::Contract::load(abi_file).unwrap(),
        );

        let bnb_text_token = "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c";
        let mut bnb_address: [u8; 20] = [0; 20];
        hex::decode_to_slice(bnb_text_token[2..].as_bytes(), &mut bnb_address).unwrap();
        let bnb_address = H160::from(bnb_address);

        let mut usdt_address: [u8; 20] = [0; 20];
        hex::decode_to_slice(usdt_token_address[2..].as_bytes(), &mut usdt_address).unwrap();
        let usdt_address = H160::from(usdt_address);

        loop {
            let amount: Vec<U256> = match contract
                .query(
                    "getAmountsOut",
                    (
                        U256::from(1000000000000000000u64),
                        vec![bnb_address, usdt_address],
                    ),
                    None,
                    Default::default(),
                    None,
                )
                .await
            {
                Ok(amount) => amount,
                Err(e) => {
                    error!("Error on getting ammounts {:?}", e);
                    sleep(Duration::from_secs(180)).await;
                    continue;
                }
            };

            let bnb_price =
                Decimal::from(amount[1].as_u128()) / Decimal::from(1000000000000000000u64);
            for token in tokens.iter() {
                let mut token_address: [u8; 20] = [0; 20];
                hex::decode_to_slice(token.contract_address[2..].as_bytes(), &mut token_address)
                    .unwrap();
                let token_address = H160::from(token_address);

                let amount: Vec<U256> = match contract
                    .query(
                        "getAmountsOut",
                        (
                            U256::from(1000000000000000000u64),
                            vec![token_address, bnb_address],
                        ),
                        None,
                        Default::default(),
                        None,
                    )
                    .await
                {
                    Ok(amount) => amount,
                    Err(e) => {
                        error!("Error on getting ammounts {:?}", e);
                        sleep(Duration::from_secs(180)).await;
                        continue;
                    }
                };

                let token_price =
                    Decimal::from(amount[1].as_u128()) / Decimal::from(1000000000000000000u64);

                let token_price = (token_price * bnb_price).to_f64().unwrap();

                if let Err(e) = db_sender.send(DbMessage::NewPrice(TokenPrice {
                    id: 0,
                    token_name: token.name.clone(),
                    price: token_price,
                })) {
                    error!(
                        "Error getting price for {:?}: {:?}",
                        token.contract_address, e
                    );
                    continue;
                }

                debug!("{:?} price: {:?}", token.name, token_price);
            }
            sleep(Duration::from_secs(180)).await;
        }
    }
}

#[allow(unused_assignments)]
pub async fn network_handler(
    network: NetworkInfo,
    rpc_urls: Vec<String>,
    games: GameInnerInfo,
    db_sender: DbSender,
    bet_sender: BetSender,
    mut last_block: Option<u64>,
) {
    //let mut restart = true;

    loop {
        //restart = false;
        let transport = rpc_urls
            .iter()
            .find_map(|url| web3::transports::Http::new(url).ok())
            .unwrap();

        debug!("Starting listening to rpc: {:?}", transport);

        let web3 = web3::Web3::new(transport);

        if last_block.is_none() {
            last_block.replace(match web3.eth().block_number().await {
                Ok(block_number) => block_number.as_u64(),
                Err(e) => {
                    error!(
                        "network id `{:?}`: Error creating filter `{:?}`",
                        network.network_id, e
                    );
                    continue;
                }
            });
        }

        let filter = FilterBuilder::default()
            .address(games.iter().map(|item| item.1 .0).collect())
            .limit(10)
            .from_block(last_block.unwrap().into())
            .build();

        // let filter_game_logs = match web3.eth_filter().create_logs_filter(filter).await {
        //     Ok(f) => f,
        //     Err(e) => {
        //         error!(
        //             "network id `{:?}`: Error creating filter `{:?}`",
        //             network.network_id, e
        //         );
        //         continue;
        //     }
        // };

        // let logs_stream = filter_game_logs.stream(time::Duration::from_secs(1));
        // futures::pin_mut!(logs_stream);

        loop {
            let logs = match web3.eth().logs(filter.clone()).await {
                Ok(logs) => logs,
                Err(e) => {
                    error!(
                        "network id `{:?}`: Error creating filter `{:?}`",
                        network.network_id, e
                    );
                    continue;
                }
            };

            debug!("Network `{}` got {} logs", network.network_id, logs.len());

            for log in logs {
                // let log = match logs_stream.next().await {
                //     Some(Ok(log)) => log,
                //     Some(Err(e)) => {
                //         error!(
                //             "Error reading log stream for Network: `{:?}` {:?}",
                //             network.network_id, e
                //         );
                //         //restart = true;
                //         break;
                //     }
                //     None => {
                //         warn!(
                //             "Connection for Network `{:?}` is closed",
                //             network.network_id
                //         );
                //         //restart = true;
                //         break;
                //     }
                // };

                handle_game_log(log, &network, &games, &db_sender, &bet_sender).await;
            }

            sleep(Duration::from_millis(10000)).await;
        }
    }
}

pub async fn bet_listener(db: DB, mut bet_receiver: BetReceiver, ws_data_feed: WsDataFeedSender) {
    while let Ok(bet) = bet_receiver.recv().await {
        if let Ok(token) = db.query_token(&bet.bet.token_address).await {
            let bet_info = BetInfoResponse {
                id: 0,
                transaction_hash: bet.bet.transaction_hash.clone(),
                player: bet.bet.player.clone(),
                player_nickname: db
                    .query_nickname(&bet.bet.player)
                    .await
                    .unwrap_or(None)
                    .map(|player| player.nickname),
                timestamp: bet.bet.timestamp,
                game_id: bet.bet.game_id,
                game_name: bet.game_name.clone(),
                wager: bet.bet.wager.clone(),
                token_address: bet.bet.token_address.clone(),
                token_name: token.name,
                network_id: bet.bet.network_id,
                network_name: bet.network_name.clone(),
                bets: bet.bet.bets,
                multiplier: bet.bet.multiplier,
                profit: bet.bet.profit.clone(),
                player_hand: bet.bet.player_hand,
            };
            if let Err(e) = ws_data_feed.send(bet_info) {
                error!("Error sending bet to the ws feed {:?}", e);
            }
        } else {
            warn!("Token `{}` not found", &bet.bet.token_address);
            let bet_info = BetInfoResponse {
                id: 0,
                transaction_hash: bet.bet.transaction_hash.clone(),
                player: bet.bet.player.clone(),
                player_nickname: db
                    .query_nickname(&bet.bet.player)
                    .await
                    .unwrap_or(None)
                    .map(|player| player.nickname),
                timestamp: bet.bet.timestamp,
                game_id: bet.bet.game_id,
                game_name: bet.game_name.clone(),
                wager: bet.bet.wager.clone(),
                token_address: bet.bet.token_address.clone(),
                token_name: "".to_string(),
                network_id: bet.bet.network_id,
                network_name: bet.network_name.clone(),
                bets: bet.bet.bets,
                multiplier: bet.bet.multiplier,
                profit: bet.bet.profit.clone(),
                player_hand: bet.bet.player_hand,
            };
            if let Err(e) = ws_data_feed.send(bet_info) {
                error!("Error sending bet to the ws feed {:?}", e);
            }
        }
    }
}

pub async fn db_listener(mut receiver: DbReceiver, db: DB) {
    while let Some(msg) = receiver.recv().await {
        match msg {
            DbMessage::PlaceBet(bet) => {
                if let Err(e) = db.place_bet(&bet).await {
                    error!("Error placing bet {:?}", e);
                }
            }
            DbMessage::NewPrice(price) => {
                if let Err(e) = db.change_token_price(&price.token_name, price.price).await {
                    error!("Error changing price {:?}: {:?}", price, e);
                }
            }
        }
    }
}
