#![allow(improper_ctypes, unused)]

#[link(wasm_import_module = "store")]
extern "C" {
    // Request stuff.
    pub fn storage(name_ptr: u32, name_len: u32) -> u32;
    pub fn size(handle: u32, key_ptr: u32, key_len: u32) -> u32;
    pub fn get(handle: u32, key_ptr: u32, key_len: u32, output_ptr: u32, output_len: u32) -> u32;
    pub fn set(handle: u32, key_ptr: u32, key_len: u32, value_ptr: u32, value_len: u32) -> ();
}
