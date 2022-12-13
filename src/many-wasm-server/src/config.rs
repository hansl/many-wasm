use crate::storage::{memory, merk, NullKvStore, StorageLibrary, StorageRef};
use many_error::ManyError;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tracing::debug;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SingleStorageConfig {
    Memory {
        name: String,
        values: Option<BTreeMap<String, String>>,
    },
    Merk {
        name: String,
        path: PathBuf,
    },
    Prefixed {
        name: String,
        prefix: String,
        backend: String,
    },
    Null {
        name: String,
    },
}

impl SingleStorageConfig {
    pub fn create_ref(
        self,
        storage: &mut StorageLibrary,
        root: impl AsRef<Path>,
        create_if_missing: bool,
    ) -> Result<(String, StorageRef), ManyError> {
        match self {
            SingleStorageConfig::Memory { name, values } => {
                debug!(r#type = "memory", name);
                let mut memory = memory::MemoryStorage::default();
                if let Some(values) = values {
                    for (key, value) in values {
                        let key = hex::decode(key).map_err(ManyError::unknown)?;
                        let value = hex::decode(value).map_err(ManyError::unknown)?;
                        memory.insert(key, value);
                    }
                }
                Ok((name, StorageRef::new(memory)))
            }
            SingleStorageConfig::Merk { name, path } => {
                let path = root.as_ref().join(path);
                debug!(
                    r#type = "merk",
                    name,
                    path = path.to_string_lossy().as_ref(),
                    create_if_missing
                );
                let merk = if create_if_missing {
                    merk::MerkStorage::new(&path, true)?
                } else {
                    merk::MerkStorage::load(&path)?
                };
                Ok((name, StorageRef::new(merk)))
            }
            SingleStorageConfig::Prefixed {
                name,
                prefix,
                backend,
            } => {
                let backend = storage
                    .get_mut(backend)
                    .ok_or_else(|| ManyError::unknown("Unknown backend storage."))?;

                Ok((name, backend.cloned_prefixed(prefix.into_bytes())))
            }
            SingleStorageConfig::Null { name } => Ok((name, StorageRef::new(NullKvStore))),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct StorageConfig(Vec<SingleStorageConfig>);

impl IntoIterator for StorageConfig {
    type Item = SingleStorageConfig;
    type IntoIter = std::vec::IntoIter<SingleStorageConfig>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Serialize, Deserialize)]
pub struct SingleModuleConfig {}

#[derive(Serialize, Deserialize)]
#[repr(transparent)]
pub struct ModuleConfig(BTreeMap<PathBuf, SingleModuleConfig>);

impl IntoIterator for ModuleConfig {
    type Item = (PathBuf, SingleModuleConfig);
    type IntoIter = std::collections::btree_map::IntoIter<PathBuf, SingleModuleConfig>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Serialize, Deserialize)]
pub struct WasmConfig {
    pub modules: ModuleConfig,
    pub storages: StorageConfig,
}
