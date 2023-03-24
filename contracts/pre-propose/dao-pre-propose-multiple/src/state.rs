use dao_voting::deposit::CheckedDepositInfo;
use cw_storage_plus::{Item};

pub const WRITE_IN_DEPOSIT_INFO: Item<Option<CheckedDepositInfo>> = Item::new("write_in_deposit_info");