use crate::abi;
use crate::config::ModuleConfig;
use crate::storage::StorageLibrary;
use abi::wasi_snapshot_preview1::create_wasi_ctx;
use anyhow::anyhow;
use many_error::ManyError;
use many_protocol::RequestMessage;
use state::WasmContext;
use std::collections::BTreeMap;
use std::path::Path;
use tracing::debug;
use wasmtime::{Engine, Linker, Module, Store};

pub mod state;

#[derive(Default)]
struct ModuleLibrary {
    endpoints: BTreeMap<String, usize>,
    modules: Vec<Module>,
}

impl ModuleLibrary {
    pub fn add(&mut self, module: Module) -> Result<(), anyhow::Error> {
        let endpoints = module
            .exports()
            .into_iter()
            .filter(|e| e.ty().func().is_some() && e.name().starts_with("endpoint "))
            .map(|e| e.name()[9..].to_string())
            .collect::<Vec<String>>();

        debug!("Adding module: endpoints = {endpoints:?}");

        for ep in endpoints.iter() {
            if self.endpoints.contains_key(ep) {
                return Err(anyhow!("Endpoint {ep} already registered."));
            }
        }

        let idx = self.modules.len();
        self.modules.push(module);
        for ep in endpoints {
            self.endpoints.insert(ep, idx);
        }

        Ok(())
    }

    pub fn get(&self, endpoint: &str) -> Option<&Module> {
        let idx = self.endpoints.get(endpoint)?;
        self.modules.get(*idx)
    }
}

pub struct WasmEngine {
    store: Store<WasmContext>,
    modules: ModuleLibrary,
    linker: Linker<WasmContext>,
}

impl WasmEngine {
    pub fn new(
        config: ModuleConfig,
        config_root: impl AsRef<Path>,
        storage: StorageLibrary,
    ) -> Result<Self, anyhow::Error> {
        let engine = Engine::default();
        let mut store = Store::new(&engine, WasmContext::new(storage, create_wasi_ctx()));
        let mut linker = Linker::new(store.engine());
        abi::link(&mut linker)?;

        let mut modules = ModuleLibrary::default();
        for (p, _c) in config {
            let wasm_path = config_root.as_ref().join(p);
            debug!(
                msg = "loading wasm",
                wasm_path = wasm_path.to_string_lossy().as_ref()
            );
            let module: Module =
                Module::from_file(store.engine(), wasm_path).map_err(|e| anyhow!("{}", e))?;

            // Instantiate at least once.
            linker
                .instantiate(&mut store, &module)
                .expect("Could not instantiate.");

            modules.add(module)?;
        }

        Ok(Self {
            store,
            modules,
            linker,
        })
    }

    pub fn call(&mut self, message: &RequestMessage) -> Result<Vec<u8>, ManyError> {
        let endpoint = message.method.to_string();
        self.store.data_mut().set_request(message.clone());

        let instance = self
            .linker
            .instantiate(
                &mut self.store,
                self.modules
                    .get(&endpoint)
                    .ok_or_else(|| ManyError::unknown("Endpoint not found"))?,
            )
            .expect("Could not instantiate.");

        let func = instance
            .get_typed_func::<(), (), _>(&mut self.store, &format!("endpoint {}", endpoint))
            .map_err(|e| ManyError::unknown(e))?;

        func.call(&mut self.store, ())
            .map_err(|e| ManyError::unknown(e))?;

        match self.store.data_mut().reset() {
            Ok(x) => x,
            Err(t) => Err(ManyError::unknown(format!("trapped: {}", t.to_string()))),
        }
    }
}
