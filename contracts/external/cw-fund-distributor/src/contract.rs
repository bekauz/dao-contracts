#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, to_binary, Uint128, WasmMsg};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TotalPowerResponse, VotingContractResponse};
use crate::state::{CW20_BALANCES, CW20_CLAIMS, DISTRIBUTION_HEIGHT, NATIVE_BALANCES, TOTAL_POWER, VOTING_CONTRACT};

use dao_interface::voting;

const CONTRACT_NAME: &str = "crates.io:cw-fund-distributor";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // store the height
    DISTRIBUTION_HEIGHT.save(deps.storage, &env.block.height)?;

    // validate the contract and save it
    let voting_contract = deps.api.addr_validate(&msg.voting_contract)?;
    VOTING_CONTRACT.save(deps.storage, &voting_contract)?;

    let total_power: voting::TotalPowerAtHeightResponse = deps.querier.query_wasm_smart(
        voting_contract.clone(),
        &voting::Query::TotalPowerAtHeight {
            height: Some(env.block.height),
        },
    )?;
    // validate the total power and store it
    if total_power.power.is_zero() {
        return Err(ContractError::ZeroVotingPower {});
    }
    TOTAL_POWER.save(deps.storage, &total_power.power)?;

    // TODO: populate ADDR_RELATIVE_SHARE map here?

    Ok(Response::default()
        .add_attribute(
            "distribution_height",
            format!("{}", env.block.height),
        )
        .add_attribute("voting_contract", voting_contract)
        .add_attribute("total_power", total_power.power)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(cw20::Cw20ReceiveMsg {
            sender: _,
            amount,
            msg: _,
        }) => execute_fund_cw20(deps, info.sender, amount),
        ExecuteMsg::FundNative {} => execute_fund_native(deps, info),
        ExecuteMsg::ClaimCW20 { tokens } => execute_claim_cw20s(deps, info.sender, tokens),
    }
}

pub fn execute_fund_cw20(
    deps: DepsMut,
    token: Addr,
    amount: Uint128
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::ZeroFunds {});
    }

    let balance = CW20_BALANCES.load(deps.storage, token.clone());
    match balance {
        Ok(old_amount) => CW20_BALANCES.save(
                deps.storage,
                token.clone(),
                &old_amount.checked_add(amount).unwrap(),
            )?,
        Err(_) => CW20_BALANCES.save(
            deps.storage,
            token.clone(),
            &amount,
            )?,
    }

    Ok(Response::default()
        .add_attribute("method", "fund_cw20")
        .add_attribute("token", token)
        .add_attribute("amount", amount)
    )
}

pub fn execute_fund_native(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut response = Response::default()
        .add_attribute("method", "fund_native");

    for Coin {amount, denom } in info.funds.into_iter() {
        match NATIVE_BALANCES.load(deps.storage, denom.clone()) {
            Ok(old_amount) => NATIVE_BALANCES.save(
                deps.storage,
                denom.clone(),
                &old_amount.checked_add(amount).unwrap(),
            ),
            Err(_) => NATIVE_BALANCES.save(
                deps.storage,
                denom.clone(),
                &amount,
            ),
        }.unwrap();
        response = response.add_attribute(denom, amount);
    };

    Ok(response)
}

pub fn execute_claim_cw20s(
    deps: DepsMut,
    sender: Addr,
    token: Option<Vec<String>>,
) -> Result<Response, ContractError> {
    // calculate the entitlement
    let voting_contract = VOTING_CONTRACT.load(deps.storage)?;
    let dist_height = DISTRIBUTION_HEIGHT.load(deps.storage)?;
    let total_power = TOTAL_POWER.load(deps.storage)?;

    let voting_power: voting::VotingPowerAtHeightResponse = deps.querier.query_wasm_smart(
        voting_contract,
        &voting::Query::VotingPowerAtHeight {
            address: sender.to_string(),
            height: Some(dist_height),
        },
    )?;

    let mut response = Response::default();
    match token {
        Some(tokens) => {
            for addr in tokens {
                // get the balance of distributor at instantiation
                let bal = CW20_BALANCES.load(
                    deps.storage,
                    Addr::unchecked(addr.clone())
                )?;

                // check for any previous claims
                let previous_claim = CW20_CLAIMS.load(
                    deps.storage,
                    (sender.clone(), Addr::unchecked(addr.clone()))
                ).unwrap_or_default();

                // get % share of sender and subtract any previous claims
                let entitlement = bal.multiply_ratio(
                voting_power.power,
                total_power
                ) - previous_claim;

                // reflect the new total claim amount
                CW20_CLAIMS.update(
                    deps.storage,
                    (sender.clone(), Addr::unchecked(addr.clone())),
                    |claim| {
                        claim.unwrap_or_default()
                            .checked_add(entitlement)
                            .map_err(StdError::overflow)
                    }
                )?;

                // add the transfer message
                response = response.add_message(WasmMsg::Execute {
                    contract_addr: addr,
                    msg: to_binary(&cw20::Cw20ExecuteMsg::Transfer {
                        recipient: sender.to_string(),
                        amount: entitlement,
                    })?,
                    funds: vec![],
                });
            }
        },
        None => {},
    }

    Ok(response
        .add_attribute("method", "claim_cw20s")
        .add_attribute("sender", sender.clone())
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VotingContract {} => query_voting_contract(deps),
        QueryMsg::TotalPower {} => query_total_power(deps),
    }
}

pub fn query_voting_contract(deps: Deps) -> StdResult<Binary> {
    let contract = VOTING_CONTRACT.load(deps.storage)?;
    let distribution_height = DISTRIBUTION_HEIGHT.load(deps.storage)?;
    to_binary(&VotingContractResponse {
        contract,
        distribution_height,
    })
}

pub fn query_total_power(deps: Deps) -> StdResult<Binary> {
    let total_power  = TOTAL_POWER.load(deps.storage)?;
    to_binary(&TotalPowerResponse {
        total_power,
    })
}