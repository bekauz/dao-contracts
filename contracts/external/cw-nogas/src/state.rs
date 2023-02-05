use cw_storage_plus::Map;

// maps the public key to nonce
pub const NONCES: Map<String, u64> = Map::new("nonces");
