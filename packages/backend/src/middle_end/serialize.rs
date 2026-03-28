//! Package which defines how a middle-end component is serialized (and deserialized)
//! into the .sim representation.

use std::path::{Path, PathBuf};
use std::fs;

use serde::{Deserialize, Serialize};
use strum::IntoDiscriminant;
use thiserror::Error;

use crate::middle_end::Wire;
use crate::middle_end::func::{Orientation, PhysicalComponentEnum, PhysicalComponentKind};

/// An error which can occur when serializing or deserializing a `.sim` file.
#[derive(Debug, Error)]
pub enum SerdeError {
    /// Error which occurs when reading a file.
    #[error("failed to read .sim file '{}': {source}", path.display())]
    ReadFile {
        /// The path which was read.
        path: PathBuf,
        /// The error.
        source: std::io::Error,
    },
    /// Error which occurs when writing a file.
    #[error("failed to write .sim file '{}': {source}", path.display())]
    WriteFile {
        /// The path which was written to.
        path: PathBuf,
        /// The error.
        source: std::io::Error,
    },
    /// Error which occurs during file deserialization
    #[error("failed to deserialize .sim JSON: {0}")]
    Deserialize(serde_json::Error),
    /// Error which occurs during file serialization.
    #[error("failed to serialize .sim JSON: {0}")]
    Serialize(serde_json::Error),
}

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
    pub name: PhysicalComponentKind,
    /// Position x.
    pub x: u32,
    /// Position y.
    pub y: u32,
    /// Properties of the component.
    pub properties: ComponentPropertiesInfo,
}

/// Serialized version of the properties of a component.
/// 
/// This is stored in the "properties" field of a [`ComponentInfo`].
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ComponentPropertiesInfo {
    /// Label.
    pub label: String,
    /// Location of label.
    pub label_location: Orientation,
    /// Internal properties of component.
    // FIXME: The component's identifier is located in [`ComponentInfo::name`],
    // and should be deserialized by using that
    #[serde(flatten)]
    pub inner: PhysicalComponentEnum,
}

impl From<super::MiddleRepr> for CircuitFile {
    fn from(value: super::MiddleRepr) -> Self {
        let version = format!("foret-{}", {
            option_env!("CARGO_PKG_VERSION")
                .unwrap_or("unknown")
        });
        Self {
            version,
            // TODO: these fields aren't currently tracked anywhere
            global_bitsize: 1,
            clock_speed: 64,
            //
            circuits: value.physical.into_iter()
                .map(|pair| pair.into())
                .collect(),
            // TODO: compute these
            revision_signatures: vec![],
        }
    }
}
impl From<(super::CircuitKey, super::CircuitArea)> for CircuitInfo {
    fn from(value: (super::CircuitKey, super::CircuitArea)) -> Self {
        let (key, super::CircuitArea { components, ui_components, wires: wire_set, .. }) = value;

        Self {
            name: key.to_string(), // TODO: assign name
            components: std::iter::chain(
                components.into_iter().map(|(_, v)| v),
                ui_components.into_iter().map(|(_, v)| v)
            ).map(Into::into).collect(),
            wires: wire_set.wires().collect()
        }
    }
}
impl From<super::ComponentProps> for ComponentInfo {
    fn from(value: super::ComponentProps) -> Self {
        let super::ComponentProps { label, label_location, origin, bounds: _, ports: _, inner } = value;

        let (x, y) = origin;
        Self {
            name: inner.discriminant(),
            x, y,
            properties: ComponentPropertiesInfo {
                label,
                label_location,
                inner,
            },
        }
    }
}

impl TryFrom<CircuitFile> for super::MiddleRepr {
    type Error = super::ReprEditErr;

    fn try_from(value: CircuitFile) -> Result<Self, Self::Error> {
        // TODO: Validate version, global_bitsize, clock_speed, each component

        let mut repr = super::MiddleRepr::new();
        for CircuitInfo { name, components, wires } in value.circuits {
            let key = repr.add_circuit();
            let mut circuit = repr.circuit(key);

            for c in components {
                let ComponentInfo { name, x, y, properties } = c;
                let ComponentPropertiesInfo { label, label_location, inner } = properties;

                circuit.add_component(inner, &label, (x, y))?;
            }

            wires.into_iter()
                .try_for_each(|w| circuit.add_wire(w))?;
        }
        Ok(repr)
    }
}
