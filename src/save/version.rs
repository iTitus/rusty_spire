use serde::de::{DeserializeOwned, IntoDeserializer};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("migration error: {0}")]
    MigrationError(#[from] MigrationError),
    #[error("expected version {expected} but got {actual}")]
    SchemaVersionMismatch { expected: i32, actual: i32 },
}

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("version did not increase from {from} to {to}")]
    NoVersionIncrease { from: i32, to: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedData {
    pub schema_version: i32,
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

pub trait Migrate: DeserializeOwned {
    const CURRENT_SCHEMA_VERSION: i32;

    fn migrate(data: VersionedData) -> Result<VersionedData, MigrationError> {
        Ok(data)
    }

    fn deserialize_and_migrate(r: impl Read) -> Result<Self, Error> {
        let mut versioned_data: VersionedData = serde_json::from_reader(r)?;
        loop {
            let version = versioned_data.schema_version;
            if version >= Self::CURRENT_SCHEMA_VERSION {
                break;
            }

            let migrated = Self::migrate(versioned_data)?;
            if migrated.schema_version <= version {
                return Err(Error::MigrationError(MigrationError::NoVersionIncrease {
                    from: version,
                    to: migrated.schema_version,
                }));
            }

            versioned_data = migrated;
        }

        if versioned_data.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(Error::SchemaVersionMismatch {
                expected: Self::CURRENT_SCHEMA_VERSION,
                actual: versioned_data.schema_version,
            });
        }

        Ok(Self::deserialize(versioned_data.data.into_deserializer())?)
    }
}
