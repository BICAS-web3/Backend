pub use tokio::sync::broadcast::{channel, Receiver, Sender};

use crate::models::db_models::{Bet, BetInfo};

pub type BetReceiver = Receiver<BetInfo>;
pub type BetSender = Sender<BetInfo>;

pub use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub type DbReceiver = UnboundedReceiver<Bet>;
pub type DbSender = UnboundedSender<Bet>;
