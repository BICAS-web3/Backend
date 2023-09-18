#[allow(unused_imports)]
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
#[allow(unused_imports)]
use utoipa_swagger_ui::Config;

use crate::handlers;
use crate::models::{db_models, json_requests, json_responses};

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
            handlers::get_bets_for_game
        ),
        components(schemas(
            json_requests::SetNickname,
            json_requests::ByNetworkId,
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
            db_models::BetInfo,
            db_models::BlockExplorerUrl,
            db_models::Game,
            db_models::GameAbi,
            db_models::NetworkInfo,
            db_models::Nickname,
            db_models::Player,
            db_models::RpcUrl,
            db_models::Token
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
