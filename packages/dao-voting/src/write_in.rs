use cosmwasm_schema::cw_serde;

use crate::multiple_choice::MultipleChoiceOption;


#[cw_serde]
pub struct WriteInMsg {
    /// The ID of the proposal to vote on.
    pub proposal_id: u64,
    /// The senders proposed voting option.
    pub write_in_vote: MultipleChoiceOption,
}
