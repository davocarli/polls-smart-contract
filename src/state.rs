use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    // The owner of the smart contract
    pub owner: Addr,
    // The question being polled
    pub question: String,
    // The options for voting
    pub options: Vec<String>,
    // Store the winner of the poll
    pub winner: Option<u8>,
}

pub const STATE: Item<Config> = Item::new("state");
pub const VOTES: Map<&Addr, Option<u8>> = Map::new("votes");
