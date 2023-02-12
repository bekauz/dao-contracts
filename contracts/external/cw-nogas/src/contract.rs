use bech32::{Variant, ToBase32};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, StdError, to_binary};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;
use cw_storage_plus::Map;
use ripemd::Ripemd160;
use sha2::{Sha256, Digest};
use crate::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ExternalMessage};
use crate::state::NONCES;
pub(crate) const CONTRACT_NAME: &str = "crates.io:cw-nogas";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("creator", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ProcessMessage { msg } => Ok(Response::default())
    }
}

pub const UNCOMPRESSED_PK_LENGTH: usize = 130;
pub const COMPRESSED_PK_LENGTH: usize = 66;

pub fn ec_pk_to_bech32_address(hex_pk: String, prefix: String) -> Result<Addr, ContractError> {

    let raw_pk = match hex_pk.clone().len() {
        // uncompressed pk: compress it and return the raw bytes
        UNCOMPRESSED_PK_LENGTH => {
            let raw_pk = hex_pk_to_raw_bytes(hex_pk.clone())?;

            // extract the compressed version of public key
            let public_key = secp256k1::PublicKey::from_slice(
                raw_pk.as_slice()
            );

            match public_key {
                Ok(pk) => pk.serialize().to_vec(),
                Err(e) => return Err(ContractError::Std(
                    StdError::GenericErr { msg: e.to_string() },
                )),
            }
        },
        // compressed pk, return the raw bytes
        COMPRESSED_PK_LENGTH => hex_pk_to_raw_bytes(hex_pk.clone())?,
        _ => return Err(ContractError::Std(
            StdError::GenericErr { 
                msg: "Unexpected hex encoded public key length".to_string()
            }
        )),
    };

    // sha256 the raw public key
    let pk_sha256 = Sha256::digest(raw_pk);

    // take the ripemd160 of the sha256 of the raw pk
    let address_raw = Ripemd160::digest(pk_sha256);
    
    // encode the prefix and the raw address bytes with Bech32
    let bech32 = bech32::encode(
        &prefix,
        address_raw.to_base32(),
        Variant::Bech32,
    );

    match bech32 {
        Ok(addr) => Ok(Addr::unchecked(addr)),
        Err(e) => Err(ContractError::Std(
            StdError::generic_err(e.to_string())
        )),
    }
}

fn hex_pk_to_raw_bytes(hex_pk: String) -> Result<Vec<u8>, ContractError> {
    let decoded_pk = hex::decode(hex_pk);
    match decoded_pk {
        Ok(pk) => Ok(pk),
        Err(e) => return Err(ContractError::Std(
            StdError::InvalidHex { msg: e.to_string() })
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}
