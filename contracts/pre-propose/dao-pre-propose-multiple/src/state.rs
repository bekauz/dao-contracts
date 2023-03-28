use dao_voting::deposit::CheckedDepositInfo;
use cw_storage_plus::{Item};

pub const WRITE_IN_DEPOSIT_INFO: Item<Option<CheckedDepositInfo>> = Item::new("write_in_deposit_info");
pub const ALLOW_WRITE_IN_VOTES: Item<bool> = Item::new("allow_write_in_votes");