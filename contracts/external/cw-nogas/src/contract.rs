use bech32::{Variant};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, StdError};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;
use cw_storage_plus::Map;
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
        ExecuteMsg::ProcessMessage { msg } => try_process_external_message(msg, deps, env, info),
    }
}

fn try_process_external_message(
    msg: ExternalMessage, 
    deps: DepsMut, 
    env: Env, 
    info: MessageInfo
) -> Result<Response, ContractError> {
    // validate the nonce
    let nonce = NONCES.load(deps.storage, msg.pk)?;
    if msg.payload.nonce != nonce {
        return Err(ContractError::InvalidNonce {  })
    }

    // validate the payload signature
    let mut sha = Sha256::new();
    sha.update(msg.payload); // TODO
    let payload_hash = sha.finalize();

    let verification = deps.api.secp256k1_verify(
        payload_hash.as_slice(), 
        &msg.signature, 
        msg.pk.as_bytes(),
    );

    match verification {
        Ok(valid_signature) => {
            // error if payload had been tampered with
            if !valid_signature {
                return Err(ContractError::InvalidSignature {  })
            }
        },
        Err(e) => return Err(ContractError::Std(StdError::from(e))),
    };

    // if message has an expiration date, validate that
    if let Some(timestamp) = msg.payload.expiration {
        if timestamp.is_expired(&env.block) {
            return Err(ContractError::ExpiredPayload {  })
        }
    }

    // bump the nonce
    NONCES.save(deps.storage, msg.pk, &(nonce + 1))?;

    // relay the message on behalf of signer
    info.sender = ec_pk_to_bech32_address(msg.pk, "juno".to_string())?;

    // execute the inner message
    let response = execute(
        deps,
        env,
        info,
        *msg.payload.msg,
    )?;
    
    Ok(Response::new()
        .add_attribute("outer_method", "try_process_external_message")
        .add_attributes(response.attributes)
    )
}

// takes an uncompressed EC public key and a prefix
pub fn ec_pk_to_bech32_address(hex_pk: String, prefix: String) -> Result<Addr, ContractError> {
    if hex_pk.clone().len() != 130 {
        return Err(ContractError::Std(
            StdError::InvalidHex {
                msg: "unexpected hex encoded uncompressed public key length".to_string()
            }
        ));
    }

    // get the raw public key bytes
    let decoded_pk = hex::decode(hex_pk);
    let raw_pk = match decoded_pk {
        Ok(pk) => pk,
        Err(e) => return Err(ContractError::Std(
            StdError::InvalidHex { msg: e.to_string() })
        ),
    };

    // extract the compressed version of public key
    let public_key = secp256k1::PublicKey::from_slice(raw_pk.as_slice());
    let raw_pk = match public_key {
        Ok(pk) => pk.serialize().to_vec(),
        Err(e) => return Err(ContractError::Std(
            StdError::GenericErr { msg: e.to_string() },
        )),
    };

    // sha256 the raw public key
    let mut hasher = Sha256::new();
    hasher.update(raw_pk);
    let pk_sha256 = hasher.finalize();

    // take the ripemd160 of the sha256 of the raw pk
    let mut hasher = Ripemd160::new();
    hasher.update(pk_sha256);
    let address_raw = hasher.finalize();

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
 
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}
