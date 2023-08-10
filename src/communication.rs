pub use tokio::sync::broadcast::{channel, Receiver, Sender};

use crate::models::db_models::Bet;

pub type BetReceiver = Receiver<Bet>;
pub type BetSender = Sender<Bet>;

pub use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub type DbReceiver = UnboundedReceiver<Bet>;
pub type DbSender = UnboundedSender<Bet>;
