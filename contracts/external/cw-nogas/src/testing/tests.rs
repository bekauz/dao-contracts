use cosmwasm_std::{StdError, Addr};

use crate::{contract::ec_pk_to_bech32_address, ContractError};


pub const JUNO_ADDRESS: &str = "juno1muw4rz9ml44wc6vssqrzkys4nuc3gylrxj4flw";
pub const COMPRESSED_PK: &str = "03f620cd2e33d3f6af5a43d5b3ca3b9b7f653aa980ae56714cc5eb7637fd1eeb28";
pub const UNCOMPRESSED_PK: &str = "04f620cd2e33d3f6af5a43d5b3ca3b9b7f653aa980ae56714cc5eb7637fd1eeb28fb722c0dacb5f005f583630dae8bbe7f5eaba70f129fc279d7ff421ae8c9eb79";
pub const PREFIX: &str = "juno";


#[test]
fn test_generate_addr_from_uncompressed_pk() -> Result<(), ContractError>{

    let generated_address = ec_pk_to_bech32_address(
        UNCOMPRESSED_PK.to_string(),
        PREFIX.to_string(),
    )?;
    assert_eq!(generated_address, Addr::unchecked(JUNO_ADDRESS));
    Ok(())
}

#[test]
fn test_generate_addr_from_compressed_pk() -> Result<(), ContractError> {

    let generated_address = ec_pk_to_bech32_address(
        COMPRESSED_PK.to_string(),
        PREFIX.to_string(),
    )?;
    assert_eq!(generated_address, Addr::unchecked(JUNO_ADDRESS));
    Ok(())
}

#[test]
fn test_generate_addr_invalid_hex_length() {
    let invalid_length_pk = "".to_string();

    let err = ec_pk_to_bech32_address(
        invalid_length_pk,
    PREFIX.to_string()
    )
    .unwrap_err();

    assert!(matches!(err, ContractError::Std(StdError::GenericErr { .. })));
}

#[test]
fn test_generate_addr_not_hex_pk() {
    let non_hex_pk = "03zzzzcd2e33d3f6af5a43d5b3ca3b9b7f653aa980ae56714cc5eb7637fd1eeb28".to_string();

    let err = ec_pk_to_bech32_address(
        non_hex_pk,
    PREFIX.to_string()
    )
    .unwrap_err();

    assert!(matches!(err, ContractError::Std(StdError::InvalidHex { .. })));
}

#[test]
fn test_generate_addr_bech32_invalid_human_readable_part() {

    let err = ec_pk_to_bech32_address(
        UNCOMPRESSED_PK.to_string(),
        "jUnO".to_string(),
    )
    .unwrap_err();

    assert!(matches!(err, ContractError::Std(StdError::GenericErr { .. })));
}

