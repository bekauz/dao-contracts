use crate::contract::ec_pk_to_bech32_address;

#[test]
fn test_generate_juno_addr_from_pk() -> Result<(), crate::ContractError>{

    let juno_address = "juno1muw4rz9ml44wc6vssqrzkys4nuc3gylrxj4flw".to_string();
    let juno_pk = "04f620cd2e33d3f6af5a43d5b3ca3b9b7f653aa980ae56714cc5eb7637fd1eeb28fb722c0dacb5f005f583630dae8bbe7f5eaba70f129fc279d7ff421ae8c9eb79".to_string();

    let generated_address = ec_pk_to_bech32_address(
        juno_pk,
        "juno".to_string()
    )?;
    assert_eq!(generated_address, juno_address);
    Ok(())
}