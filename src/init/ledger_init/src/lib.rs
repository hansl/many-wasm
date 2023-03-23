extern crate wee_alloc;

use many_identity::Address;
use many_wasm::store::Storage;
use std::collections::BTreeMap;
use std::str::FromStr;
use storage_ledger::LedgerAccount;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct LedgerConfig {
    symbols: BTreeMap<String, String>,
}

pub fn start() {
    let _ = Storage::by_name("balances");

    let _account: LedgerAccount =
        Address::from_str("maffbahksdwaqeenayy2gxke32hgb7aq4ao4wt745lsfs6wijp")
            .unwrap()
            .into();
}
