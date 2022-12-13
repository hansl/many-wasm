extern crate wee_alloc;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use many_error::ManyError;
use many_modules::ledger::{BalanceArgs, BalanceReturns};
use many_wasm::many::payload;
use many_wasm::store::Storage;
use std::collections::BTreeMap;
use storage_ledger::LedgerAccount;

#[export_name = "init"]
pub fn init() {
    // Make sure this storage is available.
    let _ = Storage::by_name("balances");
}

#[export_name = "endpoint ledger.balance"]
pub fn balance() {
    fn balance_(args: BalanceArgs) -> Result<BalanceReturns, ManyError> {
        let sender = many_wasm::many::sender();
        println!(r#"Sender: "{}""#, sender);

        let account: LedgerAccount = args.account.unwrap_or(sender).into();

        // Try to decode bytes.
        eprintln!("balance: {:?}", args);
        Ok(many_modules::ledger::BalanceReturns {
            balances: BTreeMap::new(),
        })
    }

    let args = many_wasm::many::decode(&payload()).and_then(balance_);
    match args {
        Ok(result) => many_wasm::many::set_return_data(minicbor::to_vec(result).unwrap()),
        Err(err) => many_wasm::many::set_return_error(err),
    }
}
