//! Package which defines how a middle-end component is serialized (and deserialized)
//! into the .sim representation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

use serde::de::IntoDeserializer;
use serde::{Deserialize, Serialize, Serializer};
use strum::IntoDiscriminant;
use thiserror::Error;

use crate::middle_end::{ReprEditErr, Wire};
use crate::middle_end::func::{Orientation, PComDeserCtx, PhysicalComponentEnum, PhysicalComponentKind};

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
    /// Error which occurs when constructing the middle representation.
    #[error("failed to create repr: {0}")]
    ReprCreation(ReprEditErr),
}
impl From<ReprEditErr> for SerdeError {
    fn from(value: ReprEditErr) -> Self {
        Self::ReprCreation(value)
    }
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
    #[serde(flatten)]
    pub inner: serde_json::Value,
}

impl Serialize for super::MiddleRepr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        self.to_circuit_file()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}
impl super::MiddleRepr {
    fn to_circuit_file(&self) -> Result<CircuitFile, SerdeError> {
        let version = format!("foret-{}", {
            option_env!("CARGO_PKG_VERSION")
                .unwrap_or("unknown")
        });
        
        let circuits = self.physical.values()
            .map(|area| Ok(CircuitInfo {
                name: area.name.to_string(),
                components: {
                    area.components.values()
                        .chain(area.ui_components.values())
                        .map(|props| {
                            let (x, y) = props.origin;
                            Ok(ComponentInfo {
                                name: props.inner.discriminant(),
                                x, y,
                                properties: ComponentPropertiesInfo {
                                    label: props.label.to_string(),
                                    label_location: props.label_location,
                                    inner: props.inner.serialize_with_ctx(self, serde_json::value::Serializer)?,
                                },
                            })
                        })
                        .collect::<Result<_, _>>()
                        .map_err(SerdeError::Serialize)?
                },
                wires: area.wires.wires().collect()
            }))
            .collect::<Result<_, SerdeError>>()?;

        Ok(CircuitFile {
            version,
            // TODO: these fields aren't currently tracked anywhere
            global_bitsize: 1,
            clock_speed: 64,
            //
            circuits,
            // TODO: compute these
            revision_signatures: vec![],
        })
    }
}

impl TryFrom<CircuitFile> for super::MiddleRepr {
    type Error = SerdeError;

    fn try_from(value: CircuitFile) -> Result<Self, Self::Error> {
        // TODO: Validate version, global_bitsize, clock_speed, each component

        let mut repr = super::MiddleRepr::new();
        let circuit_map: HashMap<_, _> = value.circuits.iter()
            .map(|c| (c.name.to_string(), repr.add_circuit(&c.name)))
            .collect();
        
        for CircuitInfo { name, components, wires } in value.circuits {
            let mut circuit = repr.circuit(circuit_map[&name]);

            for c in components {
                let ComponentInfo { name: kind, x, y, properties } = c;
                let ComponentPropertiesInfo { label, label_location, inner } = properties;
                
                let inner = PhysicalComponentEnum::deserialize_with_ctx(
                    PComDeserCtx { kind, circuit_map: &circuit_map },
                    inner.into_deserializer()
                ).map_err(SerdeError::Deserialize)?;
                
                circuit.add_component(inner, &label, label_location, (x, y))?;
            }

            wires.into_iter()
                .try_for_each(|w| circuit.add_wire(w))?;
        }
        Ok(repr)
    }
}

/// Applies serialization with state (context).
pub trait SerializeWithCtx<Ctx> {
    /// Serializes with the provided serializer, using the given context.
    fn serialize_with_ctx<S>(&self, ctx: &Ctx, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer;
}
impl<Ctx, T: Serialize> SerializeWithCtx<Ctx> for T {
    fn serialize_with_ctx<S>(&self, _: &Ctx, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        self.serialize(serializer)
    }
}

/// Applies deserialization with state (context).
/// 
/// Note that [`serde::DeserializeSeed`] also does deserialization with state.
/// The difference is that this trait automatically implements deserialization for all contexts
///     if the type implements [`Deserialize`],
///     which is useful for making uniform calls.
pub trait DeserializeWithCtx<'de, Ctx>: Sized {
    /// Deserializes with the provided desiralizer, using the given context.
    fn deserialize_with_ctx<D>(ctx: Ctx, deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>;
}
impl<'de, Ctx, T: Deserialize<'de>> DeserializeWithCtx<'de, Ctx> for T {
    fn deserialize_with_ctx<D>(_: Ctx, deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        Self::deserialize(deserializer)
    }
}
