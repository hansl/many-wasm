use many_identity::Address;

#[repr(C)]
pub struct LedgerAccount(Address);

impl LedgerAccount {
    pub fn balance() -> many_types::ledger::TokenAmount {
        0u32.into()
    }
}

impl Into<LedgerAccount> for Address {
    fn into(self) -> LedgerAccount {
        LedgerAccount(self)
    }
}
