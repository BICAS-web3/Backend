use serde::{Deserialize, Serialize};

pub mod db_models {
    use super::*;
    use chrono::serde::ts_seconds;
    use chrono::{DateTime, Utc};

    #[derive(Deserialize, Serialize)]
    pub struct NativeCurrency {
        pub id: i64,
        pub name: String,
        pub symbol: String,
        pub decimals: i64,
    }

    #[derive(Deserialize, Serialize)]
    pub struct Network {
        pub id: i64,
        pub name: String,
        pub native_currency_id: i64,
    }

    #[derive(Deserialize, Serialize)]
    pub struct NetworkInfo {
        pub network_id: i64,
        pub network_name: String,
        pub currency_name: String,
        pub currency_symbol: String,
        pub decimals: i64,
    }

    #[derive(Deserialize, Serialize)]
    pub struct RpcUrl {
        pub id: i64,
        pub network_id: i64,
        pub url: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct BlockExplorerUrl {
        pub id: i64,
        pub network_id: i64,
        pub url: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct Token {
        pub id: i64,
        pub network_id: i64,
        pub name: String,
        pub icon: String,
        /// 42 symbols
        pub contract_address: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct Game {
        pub id: i64,
        pub network_id: i64,
        pub name: String,
        /// 42 symbols
        pub address: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct Nickname {
        pub id: i64,
        /// 42 symbols
        pub address: String,
        pub nickname: String,
    }

    #[derive(Deserialize, Serialize)]
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

    #[derive(Deserialize, Serialize)]
    pub struct Bet {
        pub id: i64,
        pub player: String,
        #[serde(with = "ts_seconds")]
        pub timestamp: DateTime<Utc>,
        pub game_id: i64,
        pub wager: i64,
        pub token_id: i64,
        pub bets: i64,
        pub multiplier: f64,
        pub profit: f64,
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

    use super::db_models::{BlockExplorerUrl, NetworkInfo, RpcUrl, Token};
    use super::*;

    #[derive(Serialize, Deserialize)]
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

    #[derive(Serialize, Deserialize)]
    pub struct JsonResponse {
        pub status: Status,
        pub body: ResponseBody,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum ResponseBody {
        ErrorText(ErrorText),
        InfoText(InfoText),
        Networks(Networks),
        Rpcs(Rpcs),
        BlockExplorers(BlockExplorers),
        Tokens(Tokens),
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct ErrorText {
        pub error: String,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct InfoText {
        pub message: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct Networks {
        pub info: Vec<NetworkInfo>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct Rpcs {
        pub rpcs: Vec<RpcUrl>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct BlockExplorers {
        pub explorers: Vec<BlockExplorerUrl>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct Tokens {
        pub tokens: Vec<Token>,
    }
}
