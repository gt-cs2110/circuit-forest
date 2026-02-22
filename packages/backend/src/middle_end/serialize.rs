use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fmt, fs};

use serde::{Deserialize, Serialize};

use crate::middle_end::Wire;

#[derive(Debug)]
pub enum SerializeError {
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    WriteFile {
        path: PathBuf,
        source: std::io::Error,
    },
    Deserialize(serde_json::Error),
    Serialize(serde_json::Error),
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadFile { path, source } => {
                write!(f, "failed to read .sim file '{}': {source}", path.display())
            }
            Self::WriteFile { path, source } => {
                write!(
                    f,
                    "failed to write .sim file '{}': {source}",
                    path.display()
                )
            }
            Self::Deserialize(source) => write!(f, "failed to deserialize .sim JSON: {source}"),
            Self::Serialize(source) => write!(f, "failed to serialize .sim JSON: {source}"),
        }
    }
}
impl std::error::Error for SerializeError {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CircuitFile {
    /// CircuitSim version
    pub version: String,

    /// Global bit size (1-32)
    #[serde(rename = "globalBitSize")]
    pub global_bitsize: u32,

    /// Clock speed
    #[serde(rename = "clockSpeed")]
    pub clock_speed: u32,

    /// All defined circuits in this file.
    pub circuits: Vec<CircuitInfo>,

    /// A set of hashes which keeps track of all updates to the file.
    #[serde(rename = "revisionSignatures")]
    pub revision_signatures: Vec<String>,
}

impl CircuitFile {
    pub fn from_sim(s: &str) -> Result<Self, SerializeError> {
        serde_json::from_str(s).map_err(SerializeError::Deserialize)
    }
    pub fn to_sim(&self) -> Result<String, SerializeError> {
        serde_json::to_string_pretty(self).map_err(SerializeError::Serialize)
    }
    pub fn read_sim_file(path: &Path) -> Result<Self, SerializeError> {
        let s = fs::read_to_string(path).map_err(|source| SerializeError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_sim(&s)
    }
    pub fn write_sim_file(&self, path: &Path) -> Result<(), SerializeError> {
        let s = self.to_sim()?;
        fs::write(path, s).map_err(|source| SerializeError::WriteFile {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CircuitInfo {
    /// Name of the circuit.
    pub name: String,

    /// Components in the circuit.
    pub components: Vec<ComponentInfo>,

    /// Wires in circuit.
    pub wires: Vec<Wire>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ComponentInfo {
    /// Component type
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub properties: HashMap<String, String>,
}
