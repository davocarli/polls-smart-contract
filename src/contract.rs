use std::convert::TryInto;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Order, Addr};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{VotesResponse, OptionsResponse, QuestionResponse, WinnerResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, STATE, VOTES};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:polls";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.options.len() > 127 {
        return Err(ContractError::TooManyOptions {});
    }
    // Create an initial map to hold the owner as a voter
    VOTES.save(
        deps.storage,
        &info.sender,
        &None,
    )?;

    let state = Config {
        owner: info.clone().sender,
        question: msg.question.clone(),
        options: msg.options.clone(),
        winner: None,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("question", msg.question.to_string())
        .add_attribute("options", format!("{:?}", msg.options))
        .add_attribute("status", "open"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Register { address } => register_user(deps, info, address),
        ExecuteMsg::Vote { index } => try_vote(deps, info, index),
        ExecuteMsg::End {} => try_close(deps, info),
    }
}

pub fn register_user(deps: DepsMut, info: MessageInfo, address: Addr) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage).unwrap();

    if let Some(_) = state.winner {
        return Err(ContractError::PollClosed {});
    }
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }
    let current_vote = VOTES.load(deps.storage, &info.sender);

    match current_vote {
        Ok(_) => VOTES.save(deps.storage, &address, &None)?,
        Err(_) => return Err(ContractError::AlreadyRegistered {}),
    };
    Ok(Response::new().add_attribute("method", "register_user"))
}

pub fn try_vote(deps: DepsMut, info: MessageInfo, index: usize) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage).unwrap();

    if let Some(_) = state.winner {
        // Poll is closed - a winner has already been assigned
        return Err(ContractError::PollClosed {});
    }

    let current_vote = VOTES.load(deps.storage, &info.sender);
    match current_vote {
        Ok(_) => VOTES.save(
            deps.storage,
            &info.sender,
            &Some(index),
        )?,
        // Sender is not registered
        Err(_) => return Err(ContractError::NotRegistered {}),
    };
    Ok(Response::new().add_attribute("method", "try_vote"))
}

pub fn try_close(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage).unwrap();
    let mut vote_counts: Vec<u32> = vec![0; state.options.len()];
    let votes: StdResult<Vec<_>> = VOTES
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    match votes {
        Ok(all_votes) => {
            for vote in all_votes {
                if let Some(i) = vote.1 {
                    vote_counts[i] = vote_counts[i] + 1;
                }
            }
        },
        Err(_) => return Err(ContractError::DataError{}),
    }

    let max_value = vote_counts.iter().max().unwrap();
    let max_index = vote_counts.iter().position(|v| v == max_value).unwrap();
    
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if let Some(_) = state.winner {
            // Poll is closed. Winner has already been assigned.
            return Err(ContractError::PollClosed {});
        }
        if info.sender != state.owner {
            // Only the owner can close the poll
            return Err(ContractError::Unauthorized {});
        }

        state.winner = Some(max_index);

        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "try_close"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetVotes {} => to_binary(&query_votes(deps)?),
        QueryMsg::GetQuestion {} => to_binary(&query_question(deps)?),
        QueryMsg::GetOptions {} => to_binary(&query_options(deps)?),
        QueryMsg::GetWinner {} => to_binary(&query_winner(deps)?),
    }
}

fn query_votes(deps: Deps) -> StdResult<VotesResponse> {
    let votes: StdResult<Vec<_>> = VOTES
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let mut formatted_votes = vec![];

    if let Ok(all_votes) = votes {
        for vote in all_votes {
            let voted_option = vote.1;
            let address_bytes = vote.0;
            let address = String::from_utf8(address_bytes).unwrap();
            formatted_votes.push((
                address,
                voted_option,
            ));
        }
    }
    Ok(VotesResponse { votes: formatted_votes })
}

fn query_question(deps: Deps) -> StdResult<QuestionResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(QuestionResponse { question: state.question })
}

fn query_options(deps: Deps) -> StdResult<OptionsResponse> {
    let state = STATE.load(deps.storage)?;

    let mut result = vec![];

    for (idx, option) in state.options.iter().enumerate() {
        result.push((idx, option.to_owned()));
    }
    Ok(OptionsResponse { options: result })
}

fn query_winner(deps: Deps) -> StdResult<WinnerResponse> {
    let state = STATE.load(deps.storage)?;

    match state.winner {
        None => Ok(WinnerResponse { index: None, winner: None }),
        Some(index) => Ok(WinnerResponse { index: Some(index.try_into().unwrap()), winner: Some(state.options.get(index as usize).unwrap().clone())}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn initializes() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { 
            question: String::from("What's your favorite currency?"),
            options: vec![
                String::from("LUNA"),
                String::from("UST"),
                String::from("aUST"),
            ],
        };

        let info = mock_info("creator", &coins(1000, "tokens"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Query question
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetQuestion {}).unwrap();
        let value: QuestionResponse = from_binary(&res).unwrap();
        assert_eq!(value.question, String::from("What's your favorite currency?"));

        // Query options - should have LUNA, UST, and aUST as options
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOptions {}).unwrap();
        let value: OptionsResponse = from_binary(&res).unwrap();
        assert_eq!(value.options, vec![
            (0, String::from("LUNA")),
            (1, String::from("UST")),
            (2, String::from("aUST")),
        ]);

        // Query votes - should have 1 registered voter (creator) with no vote
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetVotes {}).unwrap();
        let value: VotesResponse = from_binary(&res).unwrap();
        assert_eq!(value.votes, vec![(String::from("creator"), None)]);
    }

    #[test]
    fn owner_can_vote() {
        // Instantiate smart contract
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { 
            question: String::from("What's your favorite currency?"),
            options: vec![
                String::from("LUNA"),
                String::from("UST"),
                String::from("aUST"),
            ],
        };

        let info = mock_info("creator", &coins(1000, "tokens"));

        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Owner votes
        let msg = ExecuteMsg::Vote{ index: 1 };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Query votes
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetVotes {}).unwrap();
        let value: VotesResponse = from_binary(&res).unwrap();
        assert_eq!(value.votes, vec![(String::from("creator"), Some(1))]);
    }

    #[test]
    fn owner_can_register_and_users_can_vote() {
        // Instantiate smart contract
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            question: String::from("What's your favorite currency?"),
            options: vec![
                String::from("LUNA"),
                String::from("UST"),
                String::from("aUST"),
            ],
        };

        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let voter1 = mock_info("voter1", &coins(2, "token"));
        let voter2 = mock_info("voter2", &coins(2, "token"));

        let msg = ExecuteMsg::Vote { index: 1 };
        let res = execute(deps.as_mut(), mock_env(), voter1.clone(), msg);
        if let Ok(_) = res {
            panic!("Voter 1 should not be able to vote. They have not been registered.");
        }

        // Register voter 1
        let msg = ExecuteMsg::Register { address: voter1.clone().sender };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Register voter 2
        let msg = ExecuteMsg::Register { address: voter2.clone().sender };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Query votes
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetVotes {}).unwrap();
        let value: VotesResponse = from_binary(&res).unwrap();
        assert_eq!(value.votes, vec![
            (String::from("creator"), None),
            (String::from("voter1"), None),
            (String::from("voter2"), None),
        ]);

        // Everyone votes
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), ExecuteMsg::Vote { index: 0 }).unwrap();
        let _res = execute(deps.as_mut(), mock_env(), voter1.clone(), ExecuteMsg::Vote { index: 1 }).unwrap();
        let _res = execute(deps.as_mut(), mock_env(), voter2.clone(), ExecuteMsg::Vote{ index: 0 });

        // Query the votes
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetVotes {}).unwrap();
        let value: VotesResponse = from_binary(&res).unwrap();
        assert_eq!(value.votes, vec![
            (String::from("creator"), Some(0)),
            (String::from("voter1"), Some(1)),
            (String::from("voter2"), Some(0)),
        ]);
    }

    #[test]
    fn only_owner_can_close() {
        // Instantiate smart contract
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            question: String::from("What's your favorite currency?"),
            options: vec![
                String::from("LUNA"),
                String::from("UST"),
                String::from("aUST"),
            ],
        };

        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let voter1 = mock_info("voter1", &coins(2, "token"));
        let voter2 = mock_info("voter2", &coins(2, "token"));

        // Register the voters
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), ExecuteMsg::Register { address: voter1.clone().sender });
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), ExecuteMsg::Register { address: voter2.clone().sender });

        // Everyone votes
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), ExecuteMsg::Vote { index: 0 }).unwrap();
        let _res = execute(deps.as_mut(), mock_env(), voter1.clone(), ExecuteMsg::Vote { index: 1 }).unwrap();
        let _res = execute(deps.as_mut(), mock_env(), voter2.clone(), ExecuteMsg::Vote { index: 0 }).unwrap();

        let res = execute(deps.as_mut(), mock_env(), voter1.clone(), ExecuteMsg::End {});
        if let Ok(_) = res {
            panic!("voter1 should not be able to close.");
        }

        // Query that there is no winner (yet)
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetWinner {}).unwrap();
        let value: WinnerResponse = from_binary(&res).unwrap();
        assert_eq!(value.index, None);
        assert_eq!(value.winner, None);

        // Owner closes contract
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), ExecuteMsg::End {});
        
        // Query the winner
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetWinner {}).unwrap();
        let value: WinnerResponse = from_binary(&res).unwrap();
        assert_eq!(value.index, Some(0));
        assert_eq!(value.winner, Some(String::from("LUNA")));
    }
}
