extern crate wee_alloc;

use storage_ledger::LedgerAccount;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct LedgerConfig {
    symbols: BTreeMap<String, String>,
}

#[export_name = "init"]
pub fn init() {
    let _ = Storage::by_name("balances");

    let account: LedgerAccount =
        Address::try_from("maffbahksdwaqeenayy2gxke32hgb7aq4ao4wt745lsfs6wijp")
            .unwrap()
            .into();
}
