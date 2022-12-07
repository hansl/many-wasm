#![feature(try_trait_v2)]

use crate::host::many::{error_argument, error_create, error_message};
use many_error::ManyError;
use std::ops::{ControlFlow, FromResidual, Try};

pub(crate) mod host;

pub struct ErrorResult<T> {
    inner: Result<T, ManyError>,
}

impl<T> Into<ErrorResult<T>> for Result<T, ManyError> {
    fn into(self) -> ErrorResult<T> {
        ErrorResult { inner: self }
    }
}

impl<T> FromResidual<ManyError> for ErrorResult<T> {
    fn from_residual(residual: ManyError) -> Self {
        Err(residual).into()
    }
}

impl<T> FromResidual<ErrorResult<T>> for () {
    fn from_residual(residual: ErrorResult<T>) -> Self {
        match residual.inner {
            Ok(_) => {}
            Err(err) => many::set_return_error(err),
        }
    }
}

impl<T> FromResidual<ErrorResult<T>> for ErrorResult<T> {
    fn from_residual(residual: ErrorResult<T>) -> Self {
        residual
    }
}

impl<T> Try for ErrorResult<T> {
    type Output = T;
    type Residual = ErrorResult<T>;

    fn from_output(output: Self::Output) -> Self {
        Ok(output).into()
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self.inner {
            Ok(t) => ControlFlow::Continue(t),
            Err(_) => ControlFlow::Break(self),
        }
    }
}

pub mod many {
    use super::host::many;
    use crate::{error_argument, error_create, error_message};
    use many_error::ManyError;
    use many_identity::Address;

    pub fn log(str: &str) {
        unsafe {
            many::log_str(str.as_ptr() as u32, str.len() as u32);
        }
    }

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
    pub fn decode<'a, T: minicbor::Decode<'a, ()>>(payload: &'a [u8]) -> super::ErrorResult<T> {
        let result: super::ErrorResult<T> = minicbor::decode::<'_, T>(payload)
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
