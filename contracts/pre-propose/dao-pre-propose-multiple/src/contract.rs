use cosmwasm_schema::cw_serde;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, WasmMsg, to_binary};
use cw2::set_contract_version;
use dao_interface::voting::{Query as CwCoreQuery};

use dao_pre_propose_base::{
    error::PreProposeError,
    msg::{ExecuteMsg as ExecuteBase, InstantiateMsg as InstantiateBase, QueryMsg as QueryBase},
    state::PreProposeContract,
};
use dao_voting::{multiple_choice::{MultipleChoiceOptions}};

use crate::{state::WRITE_IN_DEPOSIT_INFO, msg::{InstantiateExt, ProposeMessage, ExecuteExt, QueryExt}};

pub(crate) const CONTRACT_NAME: &str = "crates.io:dao-pre-propose-multiple";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type InstantiateMsg = InstantiateBase<InstantiateExt>;
pub type ExecuteMsg = ExecuteBase<ProposeMessage, ExecuteExt>;
pub type QueryMsg = QueryBase<QueryExt>;

use dao_voting::write_in::WriteInMsg as WriteIn;

/// Internal version of the propose message that includes the
/// `proposer` field. The module will fill this in based on the sender
/// of the external message.
#[cw_serde]
enum ProposeMessageInternal {
    Propose {
        title: String,
        description: String,
        choices: MultipleChoiceOptions,
        proposer: Option<String>,
    },
}

type PrePropose = PreProposeContract<InstantiateExt, ExecuteExt, QueryExt, ProposeMessageInternal>;
// type PrePropose = PreProposeContract<InstantiateExt, ExecuteExt, QueryExt, ProposeMessage>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, PreProposeError> {
    let dao: Addr = deps
        .querier
        .query_wasm_smart(info.sender.clone(), &CwCoreQuery::Dao {})?;

    let deposit_info = msg.clone()
        .extension
        .write_in_deposit_info
        .map(|info| info.into_checked(deps.as_ref(), dao))
        .transpose()?;

    // store the optional write_in fee
    WRITE_IN_DEPOSIT_INFO.save(deps.storage, &deposit_info)?;

    let resp = PrePropose::default().instantiate(deps.branch(), env, info, msg)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, PreProposeError> {
    // We don't want to expose the `proposer` field on the propose
    // message externally as that is to be set by this module. Here,
    // we transform an external message which omits that field into an
    // internal message which sets it.
    type ExecuteInternal = ExecuteBase<ProposeMessageInternal, ExecuteExt>;
    match msg {
        ExecuteMsg::Propose {
            msg:
                ProposeMessage::Propose {
                    title,
                    description,
                    choices,
                },
        } => PrePropose::default().execute(deps, env, info.clone(), ExecuteInternal::Propose {
            msg: ProposeMessageInternal::Propose {
                proposer: Some(info.sender.to_string()),
                title,
                description,
                choices,
            }}
        ),
        ExecuteMsg::Extension { msg } => execute_write_in_vote(deps, env, info, msg),
        ExecuteMsg::Withdraw { denom } => PrePropose::default().execute(deps, env, info, ExecuteInternal::Withdraw { denom }),
        ExecuteMsg::UpdateConfig {
            deposit_info,
            open_proposal_submission,
        } => PrePropose::default().execute(deps, env, info, ExecuteInternal::UpdateConfig {
            deposit_info,
            open_proposal_submission,
        }),
        ExecuteMsg::AddProposalSubmittedHook { address } => PrePropose::default().execute(deps, env, info,
            ExecuteInternal::AddProposalSubmittedHook { address }),
        ExecuteMsg::RemoveProposalSubmittedHook { address } => PrePropose::default().execute(deps, env, info, ExecuteInternal::RemoveProposalSubmittedHook { address }),
        ExecuteBase::ProposalCompletedHook {
            proposal_id,
            new_status,
        } => PrePropose::default().execute(deps, env, info, ExecuteInternal::ProposalCompletedHook {
            proposal_id,
            new_status,
        }),
    }
}

pub fn execute_write_in_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteExt,
) -> Result<Response, PreProposeError> {
    let proposal_module = PrePropose::default().proposal_module.load(deps.storage)?;

    let internal_message = match msg {
        ExecuteExt::WriteInVote { 
            proposal_id, 
            write_in_vote 
        } => WriteIn { 
            proposal_id, 
            write_in_vote, 
        }
    };

    let write_in_message = WasmMsg::Execute {
        contract_addr: proposal_module.into_string(),
        msg: to_binary(&internal_message)?,
        funds: info.funds,  // write in deposit
    };
    Ok(Response::default()
        .add_message(write_in_message)
        .add_attribute("method", "proposal_approved")
    )
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    PrePropose::default().query(deps, env, msg)
}
