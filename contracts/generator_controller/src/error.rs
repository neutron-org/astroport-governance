use cosmwasm_std::StdError;
use thiserror::Error;

/// This enum describes contract errors
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Basic points conversion error. {0} > 10000")]
    BPSConverstionError(u128),

    #[error("Basic points sum exceeds limit")]
    BPSLimitError {},

    #[error("You can't vote with zero voting power")]
    ZeroVotingPower {},

    #[error("{0} is the main pool. Voting for the main pool is prohibited")]
    MainPoolVoteProhibited(String),

    #[error("main_pool_min_alloc should be more than 0 and less than 1")]
    MainPoolMinAllocFailed {},

    #[error("You can only run this action every {0} days")]
    CooldownError(u64),

    #[error("Invalid lp token address: {0}")]
    InvalidLPTokenAddress(String),

    #[error("Votes contain duplicated pool addresses")]
    DuplicatedPools {},

    #[error("There are no pools to tune")]
    TuneNoPools {},

    #[error("Invalid pool number: {0}. Must be within [2, 100] range")]
    InvalidPoolNumber(u64),

    #[error("The vector contains duplicated addresses")]
    DuplicatedVoters {},

    #[error("Exceeded voters limit for kick blacklisted voters operation!")]
    KickVotersLimitExceeded {},

    #[error("Contract can't be migrated!")]
    MigrationError {},
}
