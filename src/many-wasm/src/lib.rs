#![feature(try_trait_v2)]

use crate::host::many::{error_argument, error_create, error_message};

pub(crate) mod host;

pub mod many {
    use super::host::many;
    use crate::{error_argument, error_create, error_message};
    use many_error::ManyError;
    use many_identity::Address;

    pub fn sender() -> Address {
        let mut bytes = vec![0u8; 32];

        let len = unsafe { many::sender_copy(bytes.as_mut_ptr() as u32) };
        Address::from_bytes(&bytes[..len as usize]).expect("Invalid address from host")
    }

    pub fn payload() -> Vec<u8> {
        let payload_size = unsafe { many::payload_size() };
        let bytes: Vec<u8> = vec![0u8; payload_size as usize];
        unsafe { many::payload_copy(bytes.as_ptr() as u32, payload_size) };

        bytes
    }

    /// Decode CBOR payload.
    pub fn decode<'a, T: minicbor::Decode<'a, ()>>(payload: &'a [u8]) -> Result<T, ManyError> {
        let result: Result<T, ManyError> = minicbor::decode::<'_, T>(payload)
            .map_err(|e| ManyError::deserialization_error(e))
            .into();

        result
    }

    pub fn set_return_error(err: ManyError) {
        unsafe {
            let handle = error_create(Into::<i64>::into(err.code()) as i32);
            if let Some(msg) = err.message() {
                error_message(handle, msg.as_ptr() as u32, msg.len() as u32);
            }

            for (k, v) in err.arguments() {
                error_argument(
                    handle,
                    k.as_ptr() as u32,
                    k.len() as u32,
                    v.as_ptr() as u32,
                    v.len() as u32,
                );
            }

            many::return_error(handle);
        }
    }

    pub fn set_return_data(data: Vec<u8>) {
        unsafe { many::return_data(data.as_ptr() as u32, data.len() as u32) }
    }
}

pub mod store {
    use crate::host::store;

    pub struct Storage(u32);

    impl Storage {
        pub fn by_name(name: &str) -> Self {
            let handle = unsafe { store::storage(name.as_ptr() as u32, name.len() as u32) };
            Self(handle)
        }

        pub fn get(&self, key: &[u8]) -> Vec<u8> {
            let size = unsafe { store::size(self.0, key.as_ptr() as u32, key.len() as u32) };
            let buffer: Vec<u8> = vec![0u8; size as usize];
            unsafe {
                store::get(
                    self.0,
                    key.as_ptr() as u32,
                    key.len() as u32,
                    buffer.as_ptr() as u32,
                    size,
                )
            };

            buffer
        }

        pub fn set(&self, key: &[u8], value: &[u8]) {
            unsafe {
                store::set(
                    self.0,
                    key.as_ptr() as u32,
                    key.len() as u32,
                    value.as_ptr() as u32,
                    value.len() as u32,
                );
            }
        }
    }
}
