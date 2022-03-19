use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    // The poll question to be stored on-chain
    pub question: String,
    // Vector of strings representing poll options
    pub options: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Register a new voter - can only be called by owner
    Register { address: Addr },
    // Submit a vote
    Vote { index: usize },
    // Close voting - can only be called by owner
    End  {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Get the question
    GetQuestion {},
    // Get the current vote counts
    GetVotes {},
    // Get the list of voting options
    GetOptions {},
    // Get the winner
    GetWinner {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VotesResponse {
    pub votes: Vec<(String, Option<usize>)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QuestionResponse {
    pub question: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OptionsResponse {
    pub options: Vec<(usize, String)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WinnerResponse {
    pub index: Option<u8>,
    pub winner: Option<String>,
}
