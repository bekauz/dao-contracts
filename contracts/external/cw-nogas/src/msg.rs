use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Binary};
use cw_utils::Expiration;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    ProcessMessage { msg: ExternalMessage },
}

#[cw_serde]
pub struct ExternalMessage {
    pub payload: Payload,
	pub signature: Binary,
	pub pk: String,
}

#[cw_serde]
pub struct Payload {
    pub nonce: u64,
    pub msg: Box<ExecuteMsg>,
    pub expiration: Option<Expiration>,
}

#[cw_serde]
pub enum QueryMsg {}