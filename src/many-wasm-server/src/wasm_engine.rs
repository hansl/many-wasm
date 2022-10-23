use crate::abi;
use anyhow::anyhow;
use many_error::ManyError;
use many_protocol::RequestMessage;
use std::path::Path;
use wasmtime::{Linker, Module, Store};

pub mod state;

use state::WasmState;

pub struct WasmEngine {
    store: Store<WasmState>,
    module: Module,
    linker: Linker<WasmState>,
}

impl WasmEngine {
    pub fn new<T: AsRef<Path>>(module: T) -> Result<Self, anyhow::Error> {
        let store = Store::default();
        let module: Module =
            Module::from_file(store.engine(), module).map_err(|e| anyhow!("{}", e))?;
        let mut linker = Linker::new(store.engine());
        abi::link(&mut linker)?;

        Ok(Self {
            store,
            module,
            linker,
        })
    }

    pub fn call(&mut self, message: &RequestMessage) -> Result<Vec<u8>, ManyError> {
        let endpoint = message.method.to_string();
        self.store.data_mut().set_request(message.clone());

        let instance = self
            .linker
            .instantiate(&mut self.store, &self.module)
            .expect("Could not instantiate.");

        let func = instance
            .get_typed_func::<(), (), _>(&mut self.store, &endpoint)
            .map_err(|e| ManyError::unknown(e))?;

        func.call(&mut self.store, ())
            .map_err(|e| ManyError::unknown(e))?;

        match self.store.data_mut().reset() {
            Ok(x) => x,
            Err(t) => Err(ManyError::unknown(format!("trapped: {}", t.to_string()))),
        }
    }
}
