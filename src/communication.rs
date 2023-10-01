pub use tokio::sync::broadcast::{channel, Receiver, Sender};

use crate::models::db_models::{Bet, TokenPrice};
use crate::models::json_responses::BetInfoResponse;

pub use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub enum DbMessage {
    PlaceBet(Bet),
    NewPrice(TokenPrice),
}

#[derive(Debug, Clone)]
pub struct PropagatedBet {
    pub bet: BetInfoResponse,
    pub game_name: String,
    pub network_name: String,
}

pub type DbReceiver = UnboundedReceiver<DbMessage>;
pub type DbSender = UnboundedSender<DbMessage>;

pub type BetReceiver = Receiver<PropagatedBet>;
pub type BetSender = Sender<PropagatedBet>;

pub type WsDataFeedReceiver = Receiver<BetInfoResponse>;
pub type WsDataFeedSender = Sender<BetInfoResponse>;
