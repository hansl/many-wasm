use crate::storage::{StorageLibrary, StorageRef};
use many_error::ManyError;
use many_identity::Address;
use many_protocol::RequestMessage;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU32, Ordering};
use wasmtime::Trap;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(transparent)]
pub struct RegistryHandle(u32);

static UNIQUE_ID: AtomicU32 = AtomicU32::new(0);

impl RegistryHandle {
    pub fn new() -> Self {
        Self(UNIQUE_ID.fetch_add(1, Ordering::SeqCst))
    }

    pub fn null() -> Self {
        Self(u32::MAX)
    }

    pub fn is_null(&self) -> bool {
        self.0 == u32::MAX
    }

    pub fn usize(&self) -> usize {
        self.0 as usize
    }
}

impl Into<RegistryHandle> for u32 {
    fn into(self) -> RegistryHandle {
        RegistryHandle(self)
    }
}

impl Into<u32> for RegistryHandle {
    fn into(self) -> u32 {
        self.0
    }
}

impl Into<RegistryHandle> for usize {
    fn into(self) -> RegistryHandle {
        RegistryHandle(self as u32)
    }
}

impl Into<usize> for RegistryHandle {
    fn into(self) -> usize {
        self.0 as usize
    }
}

#[non_exhaustive]
pub enum RegistryObject {
    Error(ManyError),
    Storage(StorageRef),
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
    pub fn as_storage(&self) -> Option<&StorageRef> {
        match self {
            RegistryObject::Storage(sref) => Some(sref),
            _ => None,
        }
    }
    pub fn as_storage_mut(&mut self) -> Option<&mut StorageRef> {
        match self {
            RegistryObject::Storage(sref) => Some(sref),
            _ => None,
        }
    }
}

#[derive(Default)]
struct HandleRegistry {
    inner: BTreeMap<RegistryHandle, RegistryObject>,
}

impl HandleRegistry {
    fn create(&mut self, object: RegistryObject) -> RegistryHandle {
        let handle = RegistryHandle::new();
        self.inner.insert(handle, object);
        handle
    }

    pub fn get_error(&self, handle: RegistryHandle) -> Option<&ManyError> {
        self.inner.get(&handle)?.as_error()
    }

    pub fn get_error_mut(&mut self, handle: RegistryHandle) -> Option<&mut ManyError> {
        self.inner.get_mut(&handle)?.as_error_mut()
    }

    pub fn error(&mut self, code: i32) -> RegistryHandle {
        self.create(RegistryObject::Error(ManyError::new(
            (code as i64).try_into().unwrap(),
            None,
            BTreeMap::new(),
        )))
    }

    pub fn create_storage(&mut self, storage_ref: StorageRef) -> RegistryHandle {
        self.create(RegistryObject::Storage(storage_ref))
    }

    pub fn get_storage(&mut self, handle: RegistryHandle) -> Option<&StorageRef> {
        self.inner.get(&handle)?.as_storage()
    }

    pub fn get_storage_mut(&mut self, handle: RegistryHandle) -> Option<&mut StorageRef> {
        self.inner.get_mut(&handle)?.as_storage_mut()
    }
}

pub struct WasmState {
    request: Result<RequestMessage, Trap>,
    registry: HandleRegistry,
    return_value: Option<Result<Vec<u8>, ManyError>>,
    storage_library: StorageLibrary,
}

impl WasmState {
    pub fn new(storage_library: StorageLibrary) -> Self {
        Self {
            request: Err(Trap::new("No request available in context")),
            registry: Default::default(),
            return_value: None,
            storage_library,
        }
    }

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

    pub fn get_error(&self, handle: RegistryHandle) -> Result<&ManyError, Trap> {
        self.registry
            .get_error(handle)
            .ok_or_else(|| Trap::new("invalid handle"))
    }
    pub fn get_error_mut(&mut self, handle: RegistryHandle) -> Result<&mut ManyError, Trap> {
        self.registry
            .get_error_mut(handle)
            .ok_or_else(|| Trap::new("invalid handle"))
    }
    pub fn create_error(&mut self, code: i32) -> RegistryHandle {
        self.registry.error(code)
    }

    pub fn create_storage(&mut self, name: &str) -> Option<RegistryHandle> {
        Some(
            self.registry
                .create_storage(self.storage_library.get(name)?.clone()),
        )
    }
}
