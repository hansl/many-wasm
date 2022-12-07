use crate::wasm_engine::state::WasmState;
use wasmtime::{Caller, Extern, Linker, Trap};

pub mod many;
pub mod store;

macro_rules! decl_many_imports {
    ( ($linker: ident, $mod: ident) => { $($name: ident),* $(,)? }) => {
        $(
        $linker
            .func_wrap(stringify!($mod), stringify!($name), $mod :: $name)
            .expect(concat!("Could not link fn ", stringify!($mod), " :: ", stringify!($name)));
        )*
    };
}

pub fn link(linker: &mut Linker<WasmState>) -> Result<(), Trap> {
    decl_many_imports!((linker, many) => {
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

    // decl_many_imports!((linker, store) => {
    //     new,
    //     set,
    //     get,
    // });
    Ok(())
}

// Utility functions for this module and its sub-modules.

pub(self) fn _read<R>(
    caller: &mut Caller<'_, WasmState>,
    ptr: u32,
    len: u32,
    func: impl FnOnce(&[u8]) -> Result<R, Trap>,
) -> Result<R, Trap> {
    let memory = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => return Err(Trap::new("failed to find client memory")),
    };
    let data = memory
        .data(&caller)
        .get(ptr as usize..)
        .and_then(|arr| arr.get(..len as usize))
        .ok_or_else(|| Trap::new("pointer/length out of bounds"))?;

    func(data)
}

pub(self) fn _read_str<R>(
    caller: &mut Caller<'_, WasmState>,
    ptr: u32,
    len: u32,
    func: impl FnOnce(&str) -> Result<R, Trap>,
) -> Result<R, Trap> {
    _read(caller, ptr, len, |data| {
        let s = std::str::from_utf8(data).map_err(|_| Trap::new("invalid utf-8 string"))?;
        func(s)
    })
}

pub(self) fn _store<T>(
    mut caller: Caller<WasmState>,
    ptr: u32,
    len: u32,
    func: impl FnOnce(&mut [u8]) -> Result<T, Trap>,
) -> Result<T, Trap> {
    let memory = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => return Err(Trap::new("failed to find host memory")),
    };

    let data = memory
        .data_mut(&mut caller)
        .get_mut(ptr as usize..)
        .and_then(|arr| arr.get_mut(..len as usize))
        .ok_or_else(|| Trap::new("pointer/length out of bounds"))?;
    func(data)
}
