use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod db_models {
    use super::*;
    use chrono::serde::ts_seconds;
    use chrono::{DateTime, Utc};
    use serde_with::{serde_as, DisplayFromStr};
    use sqlx::types::BigDecimal;

    #[derive(Deserialize, Serialize, ToSchema, Debug)]
    pub struct LastBlock {
        pub id: i64,
        pub network_id: i64,
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug)]
    pub struct TokenPrice {
        pub id: i64,
        pub token_name: String,
        pub price: f64,
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug)]
    pub struct PlayerTotals {
        pub bets_amount: i64,
        pub total_wagered_sum: Option<f64>,
    }

    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct Totals {
        pub bets_amount: i64,
        pub player_amount: i64,
        pub sum: Option<f64>,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct NativeCurrency {
        pub id: i64,
        pub name: String,
        pub symbol: String,
        pub decimals: i64,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct Network {
        pub id: i64,
        pub name: String,
        pub short_name: String,
        pub native_currency_id: i64,
    }

    #[derive(Deserialize, Serialize, ToSchema, Clone)]
    pub struct NetworkInfo {
        pub network_id: i64,
        pub network_name: String,
        pub short_name: String,
        pub currency_name: String,
        pub currency_symbol: String,
        pub decimals: i64,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct RpcUrl {
        pub id: i64,
        pub network_id: i64,
        pub url: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct BlockExplorerUrl {
        pub id: i64,
        pub network_id: i64,
        pub url: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct Token {
        pub id: i64,
        pub network_id: i64,
        pub name: String,
        /// 42 symbols
        pub contract_address: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct Game {
        pub id: i64,
        pub network_id: i64,
        pub name: String,
        /// 42 symbols
        pub address: String,
        pub result_event_signature: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct GameInfo {
        pub id: i64,
        pub network_id: i64,
        pub name: String,
        /// 42 symbols
        pub address: String,
        pub event_signature: String,
        pub event_types: String,
        pub event_names: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct LatestGames {
        pub name: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct Nickname {
        pub id: i64,
        /// 42 symbols
        pub address: String,
        pub nickname: String,
    }

    #[derive(Deserialize, Serialize, Default, ToSchema)]
    pub struct Player {
        pub id: i64,
        /// 42 symbols
        pub address: String,
        pub wagered: f64,
        pub bets: i64,
        pub bets_won: i64,
        pub bets_lost: i64,
        pub highest_win: f64,
        pub highest_multiplier: f64,
    }

    #[serde_as]
    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct Bet {
        pub id: i64,
        pub transaction_hash: String,
        pub player: String,
        #[serde(with = "ts_seconds")]
        pub timestamp: DateTime<Utc>,
        pub game_id: i64,
        #[serde_as(as = "DisplayFromStr")]
        pub wager: BigDecimal,
        pub token_address: String,
        pub network_id: i64,
        pub bets: i64,
        pub multiplier: f64,
        #[serde_as(as = "DisplayFromStr")]
        pub profit: BigDecimal,
        //pub player_hand: Option<[Card; 5]>,
    }

    #[serde_as]
    #[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
    pub struct BetInfo {
        pub id: i64,
        pub transaction_hash: String,
        pub player: String,
        pub player_nickname: Option<String>,
        #[serde(with = "ts_seconds")]
        pub timestamp: DateTime<Utc>,
        pub game_id: i64,
        pub game_name: String,
        #[serde_as(as = "DisplayFromStr")]
        pub wager: BigDecimal,
        pub token_address: String,
        pub token_name: String,
        pub network_id: i64,
        pub network_name: String,
        pub bets: i64,
        pub multiplier: f64,
        #[serde_as(as = "DisplayFromStr")]
        pub profit: BigDecimal,
        //pub player_hand: Option<[Card; 5]>,
    }

    #[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
    pub struct GameAbi {
        pub signature: String,
        pub types: String,
        pub names: String,
    }

    // pub struct Lobby {
    //     pub id: i64,
    //     pub game_id: i64
    // }

    // pub struct Message {
    //     // address/nickname
    //     pub from: String,
    //     pub timestamp: DateTime<Utc>,
    //     pub lobby_id:
    // }
}

pub mod json_responses {

    use super::db_models::{
        Bet, BetInfo, BlockExplorerUrl, Game, GameAbi, NetworkInfo, Nickname, Player, PlayerTotals,
        RpcUrl, Token, Totals,
    };
    use super::*;
    use chrono::serde::ts_seconds;
    use chrono::{DateTime, Utc};
    use serde_with::{serde_as, DisplayFromStr};
    use sqlx::types::BigDecimal;

    #[derive(Serialize, Deserialize, ToSchema)]
    pub enum Status {
        OK,
        Err,
    }

    #[derive(Serialize, Deserialize)]
    pub struct TextResponse {
        // OK/ERR
        //#[schema(example = "OK")]
        pub status: String,

        //#[schema(example = "Info message")]
        pub message: String,
    }

    #[derive(Serialize, Deserialize, ToSchema)]
    pub struct JsonResponse {
        pub status: Status,
        pub body: ResponseBody,
    }

    #[derive(Serialize, Deserialize, ToSchema)]
    #[serde(untagged)]
    pub enum ResponseBody {
        ErrorText(ErrorText),
        InfoText(InfoText),
        Networks(Networks),
        Rpcs(Rpcs),
        BlockExplorers(BlockExplorers),
        Tokens(Tokens),
        Game(Game),
        Nickname(Nickname),
        Player(Player),
        Bets(Bets),
        Abi(GameAbi),
        Totals(Totals),
        LatestGames(LatestGames),
        PlayerTotals(PlayerTotals),
        TokenPrice(TokenPrice),
    }

    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct ErrorText {
        pub error: String,
    }

    #[derive(Serialize, Deserialize, Clone, ToSchema)]
    pub struct InfoText {
        pub message: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct LatestGames {
        pub games: Vec<String>,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct TokenPrice {
        pub token_price: f64,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct NetworkFullInfo {
        pub basic_info: NetworkInfo,
        pub rpcs: Vec<RpcUrl>,
        pub explorers: Vec<BlockExplorerUrl>,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct Networks {
        pub networks: Vec<NetworkFullInfo>,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct Rpcs {
        pub rpcs: Vec<RpcUrl>,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct BlockExplorers {
        pub explorers: Vec<BlockExplorerUrl>,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct Tokens {
        pub tokens: Vec<Token>,
    }

    #[serde_as]
    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct Card {
        pub number: u8,
        pub suit: u8,
    }

    #[serde_as]
    #[derive(Deserialize, Serialize, ToSchema, Debug, Clone)]
    pub struct BetInfoResponse {
        pub id: i64,
        pub transaction_hash: String,
        pub player: String,
        pub player_nickname: Option<String>,
        #[serde(with = "ts_seconds")]
        pub timestamp: DateTime<Utc>,
        pub game_id: i64,
        pub game_name: String,
        #[serde_as(as = "DisplayFromStr")]
        pub wager: BigDecimal,
        pub token_address: String,
        pub token_name: String,
        pub network_id: i64,
        pub network_name: String,
        pub bets: i64,
        pub multiplier: f64,
        #[serde_as(as = "DisplayFromStr")]
        pub profit: BigDecimal,
        pub player_hand: Option<Vec<Card>>,
    }

    impl From<BetInfo> for BetInfoResponse {
        fn from(value: BetInfo) -> Self {
            BetInfoResponse {
                id: value.id,
                transaction_hash: value.transaction_hash,
                player: value.player,
                player_nickname: value.player_nickname,
                timestamp: value.timestamp,
                game_id: value.game_id,
                game_name: value.game_name,
                wager: value.wager,
                token_address: value.token_address,
                token_name: value.token_name,
                network_id: value.network_id,
                network_name: value.network_name,
                bets: value.bets,
                multiplier: value.multiplier,
                profit: value.profit,
                player_hand: None,
            }
        }
    }

    impl From<BetInfoResponse> for BetInfo {
        fn from(value: BetInfoResponse) -> Self {
            BetInfo {
                id: value.id,
                transaction_hash: value.transaction_hash,
                player: value.player,
                player_nickname: value.player_nickname,
                timestamp: value.timestamp,
                game_id: value.game_id,
                game_name: value.game_name,
                wager: value.wager,
                token_address: value.token_address,
                token_name: value.token_name,
                network_id: value.network_id,
                network_name: value.network_name,
                bets: value.bets,
                multiplier: value.multiplier,
                profit: value.profit,
            }
        }
    }

    impl From<Bet> for BetInfoResponse {
        fn from(value: Bet) -> Self {
            BetInfoResponse {
                id: value.id,
                transaction_hash: value.transaction_hash,
                player: value.player,
                player_nickname: Default::default(),
                timestamp: value.timestamp,
                game_id: value.game_id,
                game_name: Default::default(),
                wager: value.wager,
                token_address: value.token_address,
                token_name: Default::default(),
                network_id: value.network_id,
                network_name: Default::default(),
                bets: value.bets,
                multiplier: value.multiplier,
                profit: value.profit,
                player_hand: None,
            }
        }
    }
    impl From<BetInfoResponse> for Bet {
        fn from(value: BetInfoResponse) -> Self {
            Bet {
                id: value.id,
                transaction_hash: value.transaction_hash,
                player: value.player,
                timestamp: value.timestamp,
                game_id: value.game_id,
                wager: value.wager,
                token_address: value.token_address,
                network_id: value.network_id,
                bets: value.bets,
                multiplier: value.multiplier,
                profit: value.profit,
            }
        }
    }

    // #[serde_as]
    // #[derive(Deserialize, Serialize, Clone, Debug)]
    // pub struct BetIntermidiate {
    //     pub id: i64,
    //     pub transaction_hash: String,
    //     pub player: String,
    //     #[serde(with = "ts_seconds")]
    //     pub timestamp: DateTime<Utc>,
    //     pub game_id: i64,
    //     #[serde_as(as = "DisplayFromStr")]
    //     pub wager: BigDecimal,
    //     pub token_address: String,
    //     pub network_id: i64,
    //     pub bets: i64,
    //     pub multiplier: f64,
    //     #[serde_as(as = "DisplayFromStr")]
    //     pub profit: BigDecimal,
    //     //pub player_hand: Option<[Card; 5]>,
    // }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct Bets {
        pub bets: Vec<BetInfo>,
    }
}

pub mod json_requests {
    use super::*;

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct SetNickname {
        pub address: String,
        pub nickname: String,
        pub signature: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct CreateReferal {
        pub refer_to: String,
        pub referal: String,
        pub signature: String,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    pub struct ByNetworkId {
        pub network_id: i64,
    }

    #[derive(Deserialize, Serialize, ToSchema)]
    #[serde(tag = "type")]
    pub enum WebsocketsIncommingMessage {
        Subscribe { payload: Vec<String> },
        Unsubscribe { payload: Vec<String> },
        SubscribeAll,
        UnsubscribeAll,
        Ping,
    }
}
