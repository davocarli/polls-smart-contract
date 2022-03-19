use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Too many Options. Max is 127")]
    TooManyOptions {},

    #[error("User is not registered")]
    NotRegistered {},

    #[error("User is already registered")]
    AlreadyRegistered {},
    
    #[error("This poll has been closed")]
    PollClosed {},

    #[error("There was an error retrieving data")]
    DataError {}
}
