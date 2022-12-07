extern crate wee_alloc;

use many_wasm::many::payload;
use std::collections::BTreeMap;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[export_name = "init"]
pub fn init() {}

#[export_name = "endpoint ledger.balance"]
pub fn balance() {
    let sender = many_wasm::many::sender();
    many_wasm::many::log(&format!(r#"Sender: "{}""#, sender));

    // Try to decode bytes.
    let args = many_wasm::many::decode::<many_modules::ledger::BalanceArgs>(&payload())?;
    let message = format!("balance: {:?}", args);
    many_wasm::many::log(&message);

    many_wasm::many::set_return_data(
        minicbor::to_vec(many_modules::ledger::BalanceReturns {
            balances: BTreeMap::new(),
        })
        .unwrap(),
    )
}
