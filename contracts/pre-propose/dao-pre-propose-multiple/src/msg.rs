use cosmwasm_schema::cw_serde;
use dao_voting::{multiple_choice::MultipleChoiceOptions, deposit::UncheckedDepositInfo};

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
