use cosmwasm_schema::{cw_serde, QueryResponses};
use dao_voting::{multiple_choice::{MultipleChoiceOptions, MultipleChoiceOption}, deposit::UncheckedDepositInfo};

#[cw_serde]
pub enum ProposeMessage {
    Propose {
        title: String,
        description: String,
        choices: MultipleChoiceOptions,
    },
}

#[cw_serde]
pub struct InstantiateExt {
    pub write_in_deposit_info: Option<UncheckedDepositInfo>,
}

#[cw_serde]
pub enum ExecuteExt {
    /// Proposes a new voting option and casts the vote towards it.
    WriteInVote {
        /// The ID of the proposal to vote on.
        proposal_id: u64,
        /// The senders proposed voting option.
        write_in_vote: MultipleChoiceOption,
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryExt {
    /// List the approver address
    #[returns(Option<dao_voting::deposit::CheckedDepositInfo>)]
    WriteInDepositInfo {},
}