#[allow(unused_imports)]
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
#[allow(unused_imports)]
use utoipa_swagger_ui::Config;

use crate::handlers;
use crate::models::{db_models, json_requests, json_responses, LeaderboardType};

use std::sync::Arc;
use warp::{
    http::Uri,
    hyper::{Response, StatusCode},
    path::FullPath,
    path::Tail,
    Rejection, Reply,
};

#[derive(OpenApi)]
#[openapi(
        paths(
            handlers::get_networks,
            handlers::get_rpcs,
            handlers::get_block_explorers,
            handlers::get_tokens,
            handlers::get_game,
            handlers::get_nickname,
            handlers::set_nickname,
            handlers::get_player,
            handlers::get_player_bets,
            handlers::get_player_bets_inc,
            handlers::get_all_last_bets,
            handlers::get_bets_for_game,
            handlers::get_latest_games,
            handlers::get_totals,
            handlers::register_partner,
            handlers::get_partner,
            handlers::connect_wallet,
            handlers::click_partner_subid,
            handlers::add_partner_subid,
            handlers::add_partner_site,
            handlers::add_contacts,
            handlers::get_leaderboard,
            handlers::get_player_totals,
            handlers::get_partner_contacts,
            handlers::get_partner_sites,
            handlers::delete_partner_contacts,
            handlers::get_clicks,
            handlers::get_site_clicks,
            handlers::get_partner_clicks,
            handlers::get_partner_connected_wallets,
            handlers::get_partner_connected_wallets_exact_date,
            handlers::get_partner_connected_wallets_with_deposits_amount,
            handlers::get_partner_connected_wallets_info
        ),
        components(schemas(
            json_requests::SetNickname,
            json_requests::ByNetworkId,
            json_requests::RegisterPartner,
            json_requests::PartnerContactBasic,
            json_requests::AddPartnerContacts,
            json_requests::AddPartnerSite,
            json_requests::AddPartnerSubid,
            json_requests::ConnectWallet,

            json_responses::JsonResponse,
            json_responses::ResponseBody,
            json_responses::ErrorText,
            json_responses::InfoText,
            json_responses::Rpcs,
            json_responses::BlockExplorers,
            json_responses::Tokens,
            json_responses::Bets,
            json_responses::NetworkFullInfo,
            json_responses::Networks,
            json_responses::LatestGames,
            json_responses::PartnerInfo,
            json_responses::PartnerSiteInfo,

            db_models::Totals,
            db_models::BetInfo,
            db_models::BlockExplorerUrl,
            db_models::Game,
            db_models::GameAbi,
            db_models::NetworkInfo,
            db_models::Nickname,
            db_models::Player,
            db_models::RpcUrl,
            db_models::Token,
            db_models::PartnerProgram,
            db_models::Partner,
            db_models::PartnerSite,
            db_models::PartnerContact,
            db_models::SiteSubId,
            db_models::RefClicks,
            db_models::Leaderboard,
            db_models::TimeBoundaries,
            db_models::PlayerTotals,
            db_models::AmountConnectedWallets,
            db_models::ConnectedWallet,

            LeaderboardType

        )),
        tags(
            (name = "Core REST API", description = "Core REST API")
        )
    )]
pub struct ApiDoc;

pub async fn serve_swagger(
    full_path: FullPath,
    tail: Tail,
    config: Arc<Config<'static>>,
) -> Result<Box<dyn Reply + 'static>, Rejection> {
    if full_path.as_str() == "/swagger-ui" {
        return Ok(Box::new(warp::redirect::found(Uri::from_static(
            "/swagger-ui/",
        ))));
    }

    let path = tail.as_str();
    match utoipa_swagger_ui::serve(path, config) {
        Ok(file) => {
            if let Some(file) = file {
                Ok(Box::new(
                    Response::builder()
                        .header("Content-Type", file.content_type)
                        .body(file.bytes),
                ))
            } else {
                Ok(Box::new(StatusCode::NOT_FOUND))
            }
        }
        Err(error) => Ok(Box::new(
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string()),
        )),
    }
}
