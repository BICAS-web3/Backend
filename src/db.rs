use crate::{
    config::DatabaseSettings,
    models::db_models::{
        Bet, BetInfo, BlockExplorerUrl, Game, GameAbi, GameInfo, NetworkInfo, Nickname, Player,
        RpcUrl, Token,
    },
};

use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

#[derive(Debug, Clone)]
pub struct DB {
    db_pool: PgPool,
}

impl DB {
    pub async fn new(settings: &DatabaseSettings) -> Self {
        let connection_string = settings.connection_string();
        info!("Connecting to database: {}", &connection_string);

        let db_pool = PgPoolOptions::new()
            .max_connections(10)
            .connect_lazy(&connection_string)
            .expect("URI string should be correct");
        Self { db_pool }
    }

    pub async fn query_all_networks(&self) -> Result<Vec<NetworkInfo>, sqlx::Error> {
        sqlx::query_as!(
            NetworkInfo,
            r#"SELECT 
                network_id AS "network_id!",
                network_name AS "network_name!",
                short_name AS "short_name!",
                currency_name AS "currency_name!",
                currency_symbol AS "currency_symbol!",
                decimals as "decimals!"
            FROM NetworkInfo"#
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn query_token(&self, address: &str) -> Result<Token, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Token,
            r#"
            SELECT *
            FROM Token
            WHERE contract_address = $1
            LIMIT 1
            "#,
            address
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn query_all_rpcs(&self, network_id: i64) -> Result<Vec<RpcUrl>, sqlx::Error> {
        sqlx::query_as!(
            RpcUrl,
            r#"SELECT 
                id as "id!",
                network_id as "network_id!",
                url as "url!"
            FROM RpcUrl 
            WHERE network_id = $1"#,
            network_id
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn query_block_explorers(
        &self,
        network_id: i64,
    ) -> Result<Vec<BlockExplorerUrl>, sqlx::Error> {
        sqlx::query_as!(
            BlockExplorerUrl,
            r#"SELECT 
                id as "id!",
                network_id as "network_id!",
                url as "url!"
            FROM BlockExplorerUrl 
            WHERE network_id = $1"#,
            network_id
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn query_all_block_explorers(&self) -> Result<Vec<BlockExplorerUrl>, sqlx::Error> {
        sqlx::query_as!(
            BlockExplorerUrl,
            r#"SELECT 
                id as "id!",
                network_id as "network_id!",
                url as "url!"
            FROM BlockExplorerUrl"#
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn query_all_tokens(&self, network_id: i64) -> Result<Vec<Token>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Token,
            "SELECT *
            FROM Token
            WHERE network_id = $1
            ",
            network_id
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn _query_all_games_for_network(
        &self,
        network_id: i64,
    ) -> Result<Vec<Game>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Game,
            "SELECT *
            FROM Game
            WHERE network_id = $1
            ",
            network_id
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn _query_all_games(&self) -> Result<Vec<Game>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Game,
            "SELECT * 
            FROM Game
            "
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn query_game(
        &self,
        network_id: i64,
        game_name: &str,
    ) -> Result<Option<Game>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Game,
            "
            SELECT *
            FROM Game
            WHERE name = $1 
                AND network_id = $2
            LIMIT 1
            ",
            game_name,
            network_id
        )
        .fetch_optional(&self.db_pool)
        .await
    }

    pub async fn query_game_by_id(&self, game_id: i64) -> Result<Option<Game>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Game,
            "
            SELECT *
            FROM Game
            WHERE id=$1
            LIMIT 1
            ",
            game_id
        )
        .fetch_optional(&self.db_pool)
        .await
    }

    pub async fn query_all_games_infos(
        &self,
        network_id: i64,
    ) -> Result<Vec<GameInfo>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            GameInfo,
            "
            SELECT *
            FROM GameInfo
            WHERE network_id = $1
            ",
            network_id
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn query_nickname(&self, address: &str) -> Result<Option<Nickname>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Nickname,
            "
            SELECT *
            FROM Nickname
            WHERE address = $1
            LIMIT 1
            ",
            address
        )
        .fetch_optional(&self.db_pool)
        .await
    }

    pub async fn set_nickname(&self, address: &str, nickname: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO Nickname(address, nickname)
            VALUES ($1, $2)
            ON CONFLICT(address) DO UPDATE
                SET nickname = excluded.nickname
            ",
            address,
            nickname,
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn query_player(&self, address: &str) -> Result<Option<Player>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Player,
            "
            SELECT *
            FROM Player
            WHERE address = $1
            LIMIT 1
            ",
            address
        )
        .fetch_optional(&self.db_pool)
        .await
    }

    pub async fn query_bets_for_address(
        &self,
        player_address: &str,
        last_id: Option<i64>,
        page_size: i64,
    ) -> Result<Vec<BetInfo>, sqlx::Error> {
        match last_id {
            None => {
                sqlx::query_as_unchecked!(
                    BetInfo,
                    "
                SELECT *
                 FROM BetInfo
                WHERE player = $1
                ORDER BY timestamp DESC
                LIMIT $2
                ",
                    player_address,
                    page_size
                )
                .fetch_all(&self.db_pool)
                .await
            }
            Some(last_id) => {
                sqlx::query_as_unchecked!(
                    BetInfo,
                    "
                 SELECT *
                FROM BetInfo
                WHERE id < $1 AND player = $2
                ORDER BY timestamp DESC
                LIMIT $3
                ",
                    last_id,
                    player_address,
                    page_size
                )
                .fetch_all(&self.db_pool)
                .await
            }
        }
    }

    pub async fn query_bets_for_address_inc(
        &self,
        player_address: &str,
        first_id: Option<i64>,
        page_size: i64,
    ) -> Result<Vec<BetInfo>, sqlx::Error> {
        match first_id {
            None => {
                sqlx::query_as_unchecked!(
                    BetInfo,
                    "
                SELECT *
                 FROM BetInfo
                WHERE player = $1
                ORDER BY timestamp ASC
                LIMIT $2
                ",
                    player_address,
                    page_size
                )
                .fetch_all(&self.db_pool)
                .await
            }
            Some(first_id) => {
                sqlx::query_as_unchecked!(
                    BetInfo,
                    "
                 SELECT *
                FROM BetInfo
                WHERE id > $1 AND player = $2
                ORDER BY timestamp ASC
                LIMIT $3
                ",
                    first_id,
                    player_address,
                    page_size
                )
                .fetch_all(&self.db_pool)
                .await
            }
        }
    }

    pub async fn query_bets_for_game(
        &self,
        game_id: i64,
        limit: i64,
    ) -> Result<Vec<BetInfo>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            BetInfo,
            "
            SELECT *
            FROM BetInfo
            WHERE game_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
            ",
            game_id,
            limit
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn query_bets_for_game_name(
        &self,
        game_name: &str,
        limit: i64,
    ) -> Result<Vec<BetInfo>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            BetInfo,
            "
            SELECT *
            FROM BetInfo
            WHERE game_name = $1
            ORDER BY timestamp DESC
            LIMIT $2
            ",
            game_name,
            limit
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn query_bets_for_network(
        &self,
        network_id: i64,
        limit: i64,
    ) -> Result<Vec<BetInfo>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            BetInfo,
            "
            SELECT *
            FROM BetInfo
            WHERE game_id IN (
                SELECT id 
                FROM Game 
                WHERE network_id = $1)
            ORDER BY timestamp DESC
            LIMIT $2
            ",
            network_id,
            limit
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn query_all_latest_bets(&self, limit: i64) -> Result<Vec<BetInfo>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            BetInfo,
            "
            SELECT *
            FROM BetInfo
            ORDER BY timestamp DESC
            LIMIT $1
            ",
            limit
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn place_bet(&self, bet: &Bet) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO Bet(
                transaction_hash,
                player,
                timestamp,
                game_id,
                wager,
                token_address,
                network_id,
                bets,
                multiplier,
                profit
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10
            )
            ",
            bet.transaction_hash,
            bet.player,
            bet.timestamp.naive_utc(),
            bet.game_id,
            bet.wager,
            bet.token_address,
            bet.network_id,
            bet.bets,
            bet.multiplier,
            bet.profit
        )
        .execute(&self.db_pool)
        .await
        .map(|_| ())
    }

    pub async fn query_abi(&self, signature: &str) -> Result<GameAbi, sqlx::Error> {
        sqlx::query_as_unchecked!(
            GameAbi,
            "
            SELECT * FROM GameAbi WHERE signature=$1 LIMIT 1
            ",
            signature
        )
        .fetch_one(&self.db_pool)
        .await
    }
}
