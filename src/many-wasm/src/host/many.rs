#![allow(improper_ctypes, unused)]

#[link(wasm_import_module = "many")]
extern "C" {
    pub fn log_str(ptr: u32, len: u32);
    pub fn log_u32(v: u32);

    // Request stuff.
    pub fn payload_size() -> u32;
    pub fn payload_copy(ptr: u32, len: u32) -> u32;
    pub fn sender_size() -> u32;
    pub fn sender_copy(ptr: u32) -> u32;

    // Return value stuff.
    // Error.
    pub fn error_create(code: i32) -> u32;
    pub fn error_message(handle: u32, msg_ptr: u32, msg_len: u32) -> ();
    pub fn error_argument(
        handle: u32,
        key_ptr: u32,
        key_len: u32,
        value_ptr: u32,
        value_len: u32,
    ) -> ();

    pub fn return_error(id: u32) -> ();
    pub fn return_data(ptr: u32, len: u32) -> ();
}
