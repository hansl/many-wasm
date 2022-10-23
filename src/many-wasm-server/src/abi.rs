use crate::wasm_engine::state::WasmState;
use wasmtime::{Linker, Trap};

pub mod many;
pub mod store;

macro_rules! decl_many_imports {
    ( ($linker: ident, $mod: literal) => { $($name: ident),* $(,)? }) => {
        $(
        $linker
            .func_wrap($mod, stringify!($name), many::$name)
            .expect(concat!("Could not link fn ", $mod, " :: ", stringify!($name)));
        )*
    };
}

pub fn link(linker: &mut Linker<WasmState>) -> Result<(), Trap> {
    decl_many_imports!((linker, "many") => {
        log_str,
        log_u32,
        payload_size,
        payload_copy,
        sender_size,
        sender_copy,
        error_create,
        error_message,
        error_argument,
        return_error,
        return_data,
    });
    Ok(())
}
