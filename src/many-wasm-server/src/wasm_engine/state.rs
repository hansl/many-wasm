use many_error::ManyError;
use many_identity::Address;
use many_protocol::RequestMessage;
use std::collections::BTreeMap;
use wasmtime::Trap;

pub type Handle = u32;

#[non_exhaustive]
pub enum RegistryObject {
    Error(ManyError),
}

impl RegistryObject {
    pub fn as_error(&self) -> Option<&ManyError> {
        match self {
            RegistryObject::Error(e) => Some(e),
            _ => None,
        }
    }
    pub fn as_error_mut(&mut self) -> Option<&mut ManyError> {
        match self {
            RegistryObject::Error(e) => Some(e),
            _ => None,
        }
    }
}

pub struct WasmState {
    request: Result<RequestMessage, Trap>,
    registry: Vec<RegistryObject>,
    return_value: Option<Result<Vec<u8>, ManyError>>,
}

impl Default for WasmState {
    fn default() -> Self {
        Self {
            request: Err(Trap::new("No request available in context")),
            registry: Default::default(),
            return_value: None,
        }
    }
}

impl WasmState {
    pub fn reset(&mut self) -> Result<Result<Vec<u8>, ManyError>, Trap> {
        self.request = Err(Trap::new("missing context"));
        self.return_value
            .take()
            .ok_or_else(|| Trap::new("No return value was set"))
    }

    pub fn set_request(&mut self, request: RequestMessage) {
        self.request = Ok(request);
    }

    pub fn set_return_value(&mut self, value: Result<Vec<u8>, ManyError>) -> Result<(), Trap> {
        match self.return_value.replace(value) {
            None => Ok(()),
            Some(_) => Err(Trap::new("return state already set")),
        }
    }

    pub fn request(&self) -> Result<&RequestMessage, Trap> {
        self.request.as_ref().map_err(Clone::clone)
    }

    pub fn payload_size(&self) -> Result<usize, Trap> {
        Ok(self.request()?.data.len())
    }
    pub fn payload_bytes(&self) -> Result<&[u8], Trap> {
        Ok(self.request()?.data.as_slice())
    }

    pub fn sender(&self) -> Result<Address, Trap> {
        Ok(self.request()?.from.unwrap_or_default())
    }
    pub fn dest(&self) -> Result<Address, Trap> {
        Ok(self.request()?.to)
    }

    pub fn get_error(&self, handle: Handle) -> Result<&ManyError, Trap> {
        self.registry
            .get(handle as usize)
            .ok_or_else(|| Trap::new("invalid handle"))?
            .as_error()
            .ok_or_else(|| Trap::new("handle is not error"))
    }
    pub fn get_error_mut(&mut self, handle: Handle) -> Result<&mut ManyError, Trap> {
        self.registry
            .get_mut(handle as usize)
            .ok_or_else(|| Trap::new("invalid handle"))?
            .as_error_mut()
            .ok_or_else(|| Trap::new("handle is not error"))
    }
    pub fn create_error(&mut self, code: i32) -> Handle {
        self.registry.push(RegistryObject::Error(ManyError::new(
            (code as i64).try_into().unwrap(),
            None,
            BTreeMap::new(),
        )));
        (self.registry.len() - 1) as u32
    }
}
