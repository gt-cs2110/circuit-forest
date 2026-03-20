//! Package which defines how a middle-end component is serialized (and deserialized)
//! into the .sim representation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fmt, fs};

use serde::{Deserialize, Serialize};

use crate::middle_end::Wire;

/// An error which can occur when serializing or deserializing a `.sim` file.
#[derive(Debug)]
pub enum SerdeError {
    /// Error which occurs when reading a file.
    ReadFile {
        /// The path which was read.
        path: PathBuf,
        /// The error.
        source: std::io::Error,
    },
    /// Error which occurs when writing a file.
    WriteFile {
        /// The path which was written to.
        path: PathBuf,
        /// The error.
        source: std::io::Error,
    },
    /// Error which occurs during file deserialization.
    Deserialize(serde_json::Error),
    /// Error which occurs during file serialization.
    Serialize(serde_json::Error),
}

impl fmt::Display for SerdeError {
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
impl std::error::Error for SerdeError {}

/// A serialized version of the middle-end representation,
/// which is used when saving and loading from .sim files.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CircuitFile {
    /// CircuitSim version.
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
    /// Reads a string into a [`CircuitFile`].
    pub fn from_sim(s: &str) -> Result<Self, SerdeError> {
        serde_json::from_str(s).map_err(SerdeError::Deserialize)
    }
    /// Converts a [`CircuitFile`] into a string.
    pub fn to_sim(&self) -> Result<String, SerdeError> {
        serde_json::to_string_pretty(self).map_err(SerdeError::Serialize)
    }
    /// Reads code from a `.sim` file into a [`CircuitFile`].
    pub fn read_sim_file(path: &Path) -> Result<Self, SerdeError> {
        let s = fs::read_to_string(path).map_err(|source| SerdeError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_sim(&s)
    }
    /// Writes a [`CircuitFile`] into a `.sim` file.
    pub fn write_sim_file(&self, path: &Path) -> Result<(), SerdeError> {
        let s = self.to_sim()?;
        fs::write(path, s).map_err(|source| SerdeError::WriteFile {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(())
    }
}

/// Serialized version of a single circuit.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CircuitInfo {
    /// Name of the circuit.
    pub name: String,

    /// Components in the circuit.
    pub components: Vec<ComponentInfo>,

    /// Wires in circuit.
    pub wires: Vec<Wire>,
}

/// Serialized version of a component in a circuit.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ComponentInfo {
    /// Component type.
    pub name: String,
    /// Position x.
    pub x: u32,
    /// Position y.
    pub y: u32,
    /// Properties of the component.
    pub properties: HashMap<String, String>,
}
