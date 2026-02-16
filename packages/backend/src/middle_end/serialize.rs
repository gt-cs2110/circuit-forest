use std::{
    collections::HashMap,
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::middle_end::Wire;

#[derive(Debug)]
enum SerializeError {
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct CircuitFile {
    /// CircuitSim version
    version: String,

    /// Global bit size (1-32)
    #[serde(rename = "globalBitSize")]
    global_bitsize: u32,

    /// Clock speed
    #[serde(rename = "clockSpeed")]
    clock_speed: u32,

    /// All defined circuits in this file.
    circuits: Vec<CircuitInfo>,

    /// A set of hashes which keeps track of all updates to the file.
    #[serde(rename = "revisionSignatures")]
    revision_signatures: Vec<String>,
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
struct CircuitInfo {
    /// Name of the circuit.
    name: String,

    /// Components in the circuit.
    components: Vec<ComponentInfo>,

    /// Wires in circuit.
    wires: Vec<Wire>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct ComponentInfo {
    /// Component type
    name: String,
    x: u32,
    y: u32,
    properties: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test_sim() {
        let raw = include_str!("latches.sim");
        let parsed = CircuitFile::from_sim(raw).expect("Unable to deserialize");

        assert_eq!(parsed.version, "1.9.1 2110 version");
        assert_eq!(parsed.global_bitsize, 1);
        assert_eq!(parsed.clock_speed, 1);
        assert!(!parsed.circuits.is_empty());
        assert!(!parsed.revision_signatures.is_empty());
    }

    #[test]
    fn parse_test_sim_then_serialize() {
        let raw = include_str!("latches.sim");
        let parsed = CircuitFile::from_sim(raw).expect("Unable to deserialize");
        let serialized = parsed.to_sim().expect("Unable to serialize");
        let reparsed = CircuitFile::from_sim(&serialized).expect("Unable to deserialize");

        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn read_test_sim() {
        let raw = include_str!("latches.sim");
        let parsed = CircuitFile::from_sim(raw).expect("Unable to deserialize");

        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("middle_end")
            .join("latches.sim");
        let loaded = CircuitFile::read_sim_file(&path).expect("Unable to read");

        assert_eq!(parsed, loaded);
    }
}
