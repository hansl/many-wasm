use many_identity::Address;
use many_types::ledger::TokenAmount;
use many_wasm::store::Storage;

fn key_for_address_balance(address: Address, symbol: Address) -> Vec<u8> {
    format!("{}/{symbol}", address).into_bytes()
}

#[repr(C)]
pub struct LedgerAccount(Storage, Address);

impl LedgerAccount {
    pub fn new(address: Address) -> Self {
        LedgerAccount(Storage::by_name("balances"), address)
    }

    pub fn set(&self, symbol: Address, amount: TokenAmount) {
        let key = key_for_address_balance(self.1, symbol);
        self.0.set(&key, &amount.to_vec());
    }
    pub fn balance(&self, symbol: Address) -> TokenAmount {
        TokenAmount::from(self.0.get(&key_for_address_balance(self.1, symbol)))
    }
}

impl Into<LedgerAccount> for Address {
    fn into(self) -> LedgerAccount {
        LedgerAccount::new(self)
    }
}
