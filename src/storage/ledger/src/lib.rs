extern crate wee_alloc;

use many_wasm::many::payload;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[export_name = "ledger.balance"]
pub fn balance() {
    let sender = many_wasm::many::sender();
    many_wasm::many::log(&format!(r#"Sender: "{}""#, sender));

    // Try to decode bytes.
    let args = many_wasm::many::decode::<many_modules::ledger::BalanceArgs>(&payload())?;
    let message = format!("balance: {:?}", args);
    many_wasm::many::log(&message);
}
