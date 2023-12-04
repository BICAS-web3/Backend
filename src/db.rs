use crate::{
    config::DatabaseSettings,
    models::db_models::{
        AmountConnectedWallets, Bet, BetInfo, BlockExplorerUrl, Game, GameAbi, GameInfo, LastBlock,
        LatestGames, Leaderboard, NetworkInfo, Nickname, Partner, PartnerContact, PartnerProgram,
        PartnerSite, Player, PlayerTotals, RefClicks, RpcUrl, SiteSubId, TimeBoundaries, Token,
        TokenPrice, Totals,
    },
};

use chrono::{DateTime, Utc};
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

    pub async fn get_unique_tokens(&self, network_id: i64) -> Result<Vec<Token>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Token,
            r#"
            SELECT DISTINCT name,
                contract_address,
                id, 
                network_id
            FROM Token
            WHERE network_id = $1
            "#,
            network_id
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn get_latest_games(&self, address: &str) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            LatestGames,
            r#"
            SELECT game.name FROM game RIGHT JOIN 
                (SELECT * from bet where bet.player=$1 ORDER BY timestamp DESC LIMIT 2) as bets ON bets.game_id = game.id
            "#,
            address
        )
        .fetch_all(&self.db_pool)
        .await
        .map(|games| games.into_iter().map(|game| game.name).collect())
    }

    pub async fn get_totals(&self) -> Result<Totals, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Totals,
            r#"
            SELECT * FROM totals;
            "#,
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn change_token_price(
        &self,
        token_name: &str,
        new_price: f64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO TokenPrice(token_name, price)
            VALUES ($1, $2)
            ON CONFLICT(token_name) DO UPDATE
                SET price = excluded.price
            ",
            token_name,
            new_price,
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn query_token_price(
        &self,
        token_name: &str,
    ) -> Result<Option<TokenPrice>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            TokenPrice,
            r#"
            SELECT * FROM tokenprice WHERE token_name=$1 LIMIT 1
            "#,
            token_name
        )
        .fetch_optional(&self.db_pool)
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
            bet.profit,
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

    pub async fn query_player_totals(&self, address: &str) -> Result<PlayerTotals, sqlx::Error> {
        sqlx::query_as_unchecked!(
            PlayerTotals,
            r#"
            SELECT 
                    COUNT(bet.id) AS bets_amount,
                    COUNT(case when bet.wager*bet.bets > bet.profit then 1 else null end) as lost_bets,
					COUNT(case when bet.wager*bet.bets <= bet.profit then 1 else null end) as won_bets,
                    SUM((bet.wager/1000000000000000000)*bet.bets*price.price) as total_wagered_sum,
					SUM((bet.profit/1000000000000000000)*price.price) as gross_profit,
					SUM((bet.profit/1000000000000000000)*price.price)-SUM((bet.wager/1000000000000000000)*bet.bets*price.price) as net_profit,
					MAX((bet.profit/1000000000000000000)*price.price) as highest_win
            FROM bet 
			INNER JOIN (SELECT 
                                token.name AS name,
                                token.contract_address AS address,
                                tokenprice.price AS price
                        FROM token
                        INNER JOIN tokenprice ON token.name=tokenprice.token_name) AS price
              ON bet.token_address = price.address AND bet.player = $1
            "#,
            address
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn query_last_block(
        &self,
        network_id: i64,
    ) -> Result<Option<LastBlock>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            LastBlock,
            r#"
            SELECT * FROM LastBlock WHERE network_id=$1
            "#,
            network_id
        )
        .fetch_optional(&self.db_pool)
        .await
    }

    pub async fn set_last_block(&self, network_id: i64, block_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO LastBlock(id, network_id)
            VALUES ($1, $2)
            ON CONFLICT(network_id) DO UPDATE
                SET id = excluded.id
            ",
            block_id,
            network_id,
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn create_referal(&self, refer_to: &str, referal: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO Referals(
                refer_to, referal)
            VALUES ($1, $2)
            ",
            refer_to,
            referal
        )
        .execute(&self.db_pool)
        .await
        .map(|_| ())
    }

    pub async fn create_partner(
        &self,
        partner: Partner,
        contacts: &[(String, String)],
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO Partner(
                name,
                country,
                traffic_source,
                users_amount_a_month,
                main_wallet,
                program,
                is_verified
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                FALSE
            )
            "#,
            partner.name,
            partner.country,
            partner.traffic_source,
            partner.users_amount_a_month,
            partner.main_wallet,
            partner.program as PartnerProgram
        )
        .execute(&self.db_pool)
        .await?;

        self.add_partner_contacts(&partner.main_wallet, contacts)
            .await?;

        Ok(())
    }

    pub async fn get_partner(&self, wallet: &str) -> Result<Partner, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Partner,
            r#"
            SELECT * 
            FROM Partner
            WHERE main_wallet=$1
            LIMIT 1
            "#,
            wallet
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn add_partner_contacts(
        &self,
        wallet: &str,
        contacts: &[(String, String)],
    ) -> Result<(), sqlx::Error> {
        for (name, url) in contacts {
            sqlx::query!(
                r#"
                INSERT INTO PartnerContact(
                    name,
                    url,
                    partner_id
                ) VALUES (
                    $1,
                    $2,
                    $3
                )
                "#,
                name,
                url,
                wallet
            )
            .execute(&self.db_pool)
            .await?;
        }

        Ok(())
    }

    pub async fn delete_partner_contacts(
        &self,
        wallet: &str,
        contact_ids: &[i64],
    ) -> Result<(), sqlx::Error> {
        for contact_id in contact_ids.iter() {
            sqlx::query!(
                r#"
                DELETE FROM partnercontact where id = $1 AND partner_id = $2
                "#,
                contact_id,
                wallet
            )
            .execute(&self.db_pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_partner_contacts(
        &self,
        wallet: &str,
    ) -> Result<Vec<PartnerContact>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            PartnerContact,
            r#"
            SELECT * 
            FROM PartnerContact
            WHERE partner_id=$1
            "#,
            wallet
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn add_partner_site(
        &self,
        wallet: &str,
        url: &str,
        name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO PartnerSite(
                id,
                name,
                url,
                partner_id
            ) 
            SELECT 
                COALESCE(MAX(id)+1,0),
                $1,
                $2,
                $3
            FROM PartnerSite
            WHERE partner_id=$3
            "#,
            name,
            url,
            wallet
        )
        .execute(&self.db_pool)
        .await
        .map(|_| ())
    }

    pub async fn get_partner_sites(&self, wallet: &str) -> Result<Vec<PartnerSite>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            PartnerSite,
            r#"
            SELECT *
            FROM PartnerSite
            WHERE partner_id=$1
            "#,
            wallet
        )
        .fetch_all(&self.db_pool)
        .await
    }

    // pub async fn get_partner_site(&self, wallet: &str) -> Result<PartnerSite, sqlx::Error> {
    //     sqlx::query_as_unchecked!(
    //         PartnerSite,
    //         r#"
    //         SELECT *
    //         FROM PartnerSite
    //         WHERE partner_id=$1
    //         LIMIT 1
    //         "#,
    //         wallet
    //     )
    //     .fetch_one(&self.db_pool)
    //     .await
    // }

    pub async fn add_partner_subid(
        &self,
        internal_site_id: i64,
        wallet: &str,
        url: &str,
        name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO SiteSubId(
                id,
                name,
                url,
                site_id,
                partner_id
            ) 
            SELECT 
                COALESCE(MAX(id)+1,0),
                $1,
                $2,
                $3,
                $4
            FROM SiteSubId
            WHERE site_id=$3
            "#,
            name,
            url,
            internal_site_id,
            wallet
        )
        .execute(&self.db_pool)
        .await
        .map(|_| ())
    }

    pub async fn get_site_subids(
        &self,
        internal_site_id: i64,
    ) -> Result<Vec<SiteSubId>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            SiteSubId,
            r#"
            SELECT *
            FROM SiteSubId
            WHERE site_id=$1
            "#,
            internal_site_id
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn get_subid(
        &self,
        wallet: &str,
        site_id: i64,
        sub_id: i64,
    ) -> Result<SiteSubId, sqlx::Error> {
        sqlx::query_as_unchecked!(
            SiteSubId,
            r#"
            SELECT 
                sitesubid.internal_id,
                sitesubid.id,
                sitesubid.name,
                sitesubid.url,
                sitesubid.site_id,
                sitesubid.partner_id
            FROM partnersite 
            INNER JOIN sitesubid ON site_id=partnersite.internal_id AND partnersite.partner_id=sitesubid.partner_id
            WHERE partnersite.partner_id=$1 AND partnersite.id=$2 AND sitesubid.id=$3
            "#,
            wallet,
            site_id,
            sub_id
        ).fetch_one(&self.db_pool)
        .await
    }

    pub async fn get_subid_clicks(
        &self,
        partner: &str,
        site_id: i64,
        sub_id: i64,
    ) -> Result<RefClicks, sqlx::Error> {
        sqlx::query_as_unchecked!(
            RefClicks,
            r#"
            SELECT 
                refclicks.id,
                refclicks.clicks,
                refclicks.sub_id_internal,
                refclicks.partner_id
            FROM refclicks
            INNER JOIN (SELECT 
                sitesubid.internal_id
            FROM partnersite 
            INNER JOIN sitesubid ON site_id=partnersite.internal_id AND partnersite.partner_id=sitesubid.partner_id
            WHERE partnersite.partner_id=$1 
                        AND partnersite.id=$2 
                        AND sitesubid.id=$3) AS subids ON subids.internal_id=refclicks.sub_id_internal;
            "#,
            partner,
            site_id,
            sub_id
        ).fetch_one(&self.db_pool)
        .await
    }

    pub async fn get_site_clicks(
        &self,
        partner: &str,
        site_id: i64,
    ) -> Result<RefClicks, sqlx::Error> {
        sqlx::query_as_unchecked!(
            RefClicks,
            r#"
            SELECT 
                0 AS id,
                COALESCE(SUM(clicks.clicks), 0) AS clicks,
                0 AS sub_id_internal,
                $1 AS partner_id
            FROM partnersite
            INNER JOIN (SELECT * FROM refclicks
                    INNER JOIN sitesubid ON sitesubid.internal_id=refclicks.sub_id_internal
                    WHERE refclicks.partner_id=$1) as clicks
            ON partnersite.internal_id=clicks.site_id
            WHERE partnersite.id = $2;
            "#,
            partner,
            site_id
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn get_partner_clicks(&self, partner: &str) -> Result<RefClicks, sqlx::Error> {
        sqlx::query_as_unchecked!(
            RefClicks,
            r#"
            SELECT 
                CAST(0 as bigint) AS id,
                CAST(COALESCE(SUM(clicks.clicks), 0) as BIGINT) AS clicks,
                CAST(0 as bigint) AS sub_id_internal,
                $1 AS partner_id
            FROM partnersite
            INNER JOIN (SELECT * FROM refclicks
                    INNER JOIN sitesubid ON sitesubid.internal_id=refclicks.sub_id_internal
                    WHERE refclicks.partner_id=$1) as clicks
            ON partnersite.internal_id=clicks.site_id;
            "#,
            partner
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn get_partner_connected_wallets_amount(
        &self,
        partner: &str,
    ) -> Result<AmountConnectedWallets, sqlx::Error> {
        sqlx::query_as_unchecked!(
            AmountConnectedWallets,
            r#"
            SELECT COUNT(connectedwallets.address) as connected_wallets FROM connectedwallets WHERE partner_id=$1
            "#,
            partner
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn add_click(&self, partner: &str, sub_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO refclicks(
                clicks,
                sub_id_internal,
                partner_id
            )
             VALUES (
                 1, 
                 $1,
                 $2
             )
             ON CONFLICT(sub_id_internal,partner_id) DO UPDATE
             SET clicks = refclicks.clicks+1;
            "#,
            sub_id,
            partner
        )
        .execute(&self.db_pool)
        .await
        .map(|_| ())
    }

    pub async fn add_ref_wallet(
        &self,
        address: &str,
        timestamp: DateTime<Utc>,
        sub_id_internal: i64,
        partner_wallet: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO ConnectedWallets(
                address,
                timestamp,
                sub_id_internal,
                partner_id
            ) VALUES(
                $1,
                $2,
                $3,
                $4
            )
            "#,
            address,
            timestamp.naive_utc(),
            sub_id_internal,
            partner_wallet
        )
        .execute(&self.db_pool)
        .await
        .map(|_| ())
    }

    pub async fn query_leaderboard_volume(
        &self,
        time_boundaries: TimeBoundaries,
        limit: i64,
    ) -> Result<Vec<Leaderboard>, sqlx::Error> {
        match time_boundaries {
            TimeBoundaries::Daily => {
                sqlx::query_as_unchecked!(
                    Leaderboard,
                    r#"
                SELECT bet.player, bet.total, nickname.nickname from (
                    SELECT 
                        bet.player,
                        SUM((bet.wager/1000000000000000000)*bet.bets*price.price) as total
                    FROM bet
                    INNER JOIN (SELECT 
                                    token.name AS name,
                                    token.contract_address AS address,
                                    tokenprice.price AS price
                                FROM token
                                INNER JOIN tokenprice ON token.name=tokenprice.token_name) AS price
                    ON bet.token_address = price.address
                    WHERE bet.timestamp > now() - interval '1 day'
                    GROUP BY bet.player) as bet
                    LEFT JOIN nickname ON bet.player=nickname.address
                    ORDER BY total DESC
                LIMIT $1;
                "#,
                    limit
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::Weekly => {
                sqlx::query_as_unchecked!(
                    Leaderboard,
                    r#"
                SELECT bet.player, bet.total, nickname.nickname from (
                    SELECT 
                        bet.player,
                        SUM((bet.wager/1000000000000000000)*bet.bets*price.price) as total
                    FROM bet
                    INNER JOIN (SELECT 
                                    token.name AS name,
                                    token.contract_address AS address,
                                    tokenprice.price AS price
                                FROM token
                                INNER JOIN tokenprice ON token.name=tokenprice.token_name) AS price
                    ON bet.token_address = price.address
                    WHERE bet.timestamp > now() - interval '1 week'
                    GROUP BY bet.player) as bet
                    LEFT JOIN nickname ON bet.player=nickname.address
                    ORDER BY total DESC
                LIMIT $1;
                "#,
                    limit
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::Monthly => {
                sqlx::query_as_unchecked!(
                    Leaderboard,
                    r#"
                SELECT bet.player, bet.total, nickname.nickname from (
                    SELECT 
                        bet.player,
                        SUM((bet.wager/1000000000000000000)*bet.bets*price.price) as total
                    FROM bet
                    INNER JOIN (SELECT 
                                    token.name AS name,
                                    token.contract_address AS address,
                                    tokenprice.price AS price
                                FROM token
                                INNER JOIN tokenprice ON token.name=tokenprice.token_name) AS price
                    ON bet.token_address = price.address
                    WHERE bet.timestamp > now() - interval '1 month'
                    GROUP BY bet.player) as bet
                    LEFT JOIN nickname ON bet.player=nickname.address
                    ORDER BY total DESC
                LIMIT $1;
                "#,
                    limit
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::All => {
                sqlx::query_as_unchecked!(
                    Leaderboard,
                    r#"
                SELECT bet.player, bet.total, nickname.nickname from (
                    SELECT 
                        bet.player,
                        SUM((bet.wager/1000000000000000000)*bet.bets*price.price) as total
                    FROM bet
                    INNER JOIN (SELECT 
                                    token.name AS name,
                                    token.contract_address AS address,
                                    tokenprice.price AS price
                                FROM token
                                INNER JOIN tokenprice ON token.name=tokenprice.token_name) AS price
                    ON bet.token_address = price.address
                    GROUP BY bet.player) as bet
                    LEFT JOIN nickname ON bet.player=nickname.address
                    ORDER BY total DESC
                LIMIT $1;
                "#,
                    limit
                )
                .fetch_all(&self.db_pool)
                .await
            }
        }
    }

    pub async fn query_leaderboard_profit(
        &self,
        time_boundaries: TimeBoundaries,
        limit: i64,
    ) -> Result<Vec<Leaderboard>, sqlx::Error> {
        match time_boundaries {
            TimeBoundaries::Daily => {
                sqlx::query_as_unchecked!(
                    Leaderboard,
                    r#"
                SELECT bet.player, bet.total, nickname.nickname from (
                    SELECT 
                        bet.player,
                        SUM((bet.profit/1000000000000000000)*price.price) as total
                    FROM bet
                    INNER JOIN (SELECT 
                                    token.name AS name,
                                    token.contract_address AS address,
                                    tokenprice.price AS price
                                FROM token
                                INNER JOIN tokenprice ON token.name=tokenprice.token_name) AS price
                    ON bet.token_address = price.address
                    WHERE bet.timestamp > now() - interval '1 day'
                    GROUP BY bet.player) as bet
                    LEFT JOIN nickname ON bet.player=nickname.address
                    ORDER BY total DESC
                LIMIT $1;
                "#,
                    limit
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::Weekly => {
                sqlx::query_as_unchecked!(
                    Leaderboard,
                    r#"
                SELECT bet.player, bet.total, nickname.nickname from (
                    SELECT 
                        bet.player,
                        SUM((bet.profit/1000000000000000000)*price.price) as total
                    FROM bet
                    INNER JOIN (SELECT 
                                    token.name AS name,
                                    token.contract_address AS address,
                                    tokenprice.price AS price
                                FROM token
                                INNER JOIN tokenprice ON token.name=tokenprice.token_name) AS price
                    ON bet.token_address = price.address
                    WHERE bet.timestamp > now() - interval '1 week'
                    GROUP BY bet.player) as bet
                    LEFT JOIN nickname ON bet.player=nickname.address
                    ORDER BY total DESC
                LIMIT $1;
                "#,
                    limit
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::Monthly => {
                sqlx::query_as_unchecked!(
                    Leaderboard,
                    r#"
                SELECT bet.player, bet.total, nickname.nickname from (
                    SELECT 
                        bet.player,
                        SUM((bet.profit/1000000000000000000)*price.price) as total
                    FROM bet
                    INNER JOIN (SELECT 
                                    token.name AS name,
                                    token.contract_address AS address,
                                    tokenprice.price AS price
                                FROM token
                                INNER JOIN tokenprice ON token.name=tokenprice.token_name) AS price
                    ON bet.token_address = price.address
                    WHERE bet.timestamp > now() - interval '1 month'
                    GROUP BY bet.player) as bet
                    LEFT JOIN nickname ON bet.player=nickname.address
                    ORDER BY total DESC
                LIMIT $1;
                "#,
                    limit
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::All => {
                sqlx::query_as_unchecked!(
                    Leaderboard,
                    r#"
                SELECT bet.player, bet.total, nickname.nickname from (
                    SELECT 
                        bet.player,
                        SUM((bet.profit/1000000000000000000)*price.price) as total
                    FROM bet
                    INNER JOIN (SELECT 
                                    token.name AS name,
                                    token.contract_address AS address,
                                    tokenprice.price AS price
                                FROM token
                                INNER JOIN tokenprice ON token.name=tokenprice.token_name) AS price
                    ON bet.token_address = price.address
                    GROUP BY bet.player) as bet
                    LEFT JOIN nickname ON bet.player=nickname.address
                    ORDER BY total DESC
                LIMIT $1;
                "#,
                    limit
                )
                .fetch_all(&self.db_pool)
                .await
            }
        }
    }
}
