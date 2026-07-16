use std::io::Split;

use crate::bitarray::BitArray;
use crate::engine::func::{self, BitSize, SplitterConfig};
use crate::bitarr;
use crate::middle_end::func::{Handedness, Orientation, PhysicalComponent, PhysicalInitContext, RelativeComponentBounds};

/// An input.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pin {
    bitsize: BitSize,
    is_input: bool,
    orientation: Orientation
}
impl Pin{
    /// Creates a new instance of the pin with specified bitsize and whether it's an input or output.
    pub fn new(bitsize: u8, is_input: bool, orientation: Orientation) -> Self {
        Self {
            bitsize: BitSize::new_clamped(bitsize),
            is_input,
            orientation
        }
    }
}
impl PhysicalComponent for Pin {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(match self.is_input {
            true  => func::Input::new(self.bitsize.get()).into(),
            false => func::Output::new(self.bitsize.get()).into()
        })
    }

    fn component_name(&self) -> &'static str {
        match self.is_input {
            true  => "Input",
            false => "Output"
        }
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_from_bitsize(self.bitsize.get(), self.orientation)
    }
}

/// A constant.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Constant {
    value: BitArray,
    orientation: Orientation
}
impl Constant {
    /// Creates a new instance of the constant with specified value.
    pub fn new(value: BitArray, orientation: Orientation) -> Self {
        Self { value, orientation }
    }

    /// Gets the value associated with this constant.
    pub fn get_value(&self) -> BitArray {
        self.value
    }
}

impl PhysicalComponent for Constant {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::Constant::new(self.value).into())
    }

    fn component_name(&self) -> &'static str {
        "Constant"
    }
    

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_from_bitsize(std::cmp::max(self.value.len(), 2), self.orientation)
    }
}

/// Power (essentially a constant 1).
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Power;
impl PhysicalComponent for Power {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::Constant::new(bitarr![1]).into())
    }

    fn component_name(&self) -> &'static str {
        "Power"
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_with_origin(2, 3, (1, 3))
    }
}

/// Ground (essentially a constant 0).
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ground;
impl PhysicalComponent for Ground {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::Constant::new(bitarr![0]).into())
    }

    fn component_name(&self) -> &'static str {
        "Ground"
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_with_origin(2, 3, (1, 0))
    }
}

/// A splitter component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Splitter {
    config:SplitterConfig,
    orientation:Orientation,
    handedness:Handedness,
}
impl Splitter {
    /// Creates a new instance of the splitter with specified bitsize.
    pub fn new(port_assignments: [Option<u8>; 64], num_legs: u8, bitsize:u8, orientation:Orientation, handedness:Handedness) -> Self {
        
        Self {
             config:SplitterConfig::new(port_assignments, num_legs,bitsize).unwrap(),orientation,handedness
        }
    }
    
}
impl PhysicalComponent for Splitter {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::Splitter::new(self.config).into())
    }

    fn component_name(&self) -> &'static str {
        "Splitter"
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        let bitsize = i32::from(self.config.get_bitsize().get());
        let mut ports = vec![(0, 0)];
        ports.extend((1..=bitsize).map(|i| (2 * i, 2)));

        RelativeComponentBounds::new((bitsize * 2, 2), ports)
            .orient(self.orientation, self.handedness)
    }
}

/// A tunnel. TODO names for linking tunnels
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tunnel {
    orientation: Orientation,
}
impl Tunnel {
    /// Creates a new instance of a tunnel with the specified orientation.
    pub fn new(orientation: Orientation) -> Self {
        Self { orientation }
    }
}
impl PhysicalComponent for Tunnel {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        None
    }

    fn component_name(&self) ->  &'static str {
        "Tunnel"
    }

    fn init_bounds(&self, ctx: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        let width = std::cmp::max(2, ctx.label.len() as u32).next_multiple_of(2);
        RelativeComponentBounds::single_port(width, 2, self.orientation)
    }
}

/// A probe.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Probe {
    orientation: Orientation,
    bitsize: BitSize
}
impl Probe {
    /// Creates a new instance of the probe with specified orientation.
    pub fn new(orientation: Orientation, bitsize: u8) -> Self {
        Self { orientation, bitsize: BitSize::new_clamped(bitsize) }
    }
}
impl PhysicalComponent for Probe {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        None
    }

    fn component_name(&self) ->  &'static str {
        "Probe"
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_from_bitsize(self.bitsize.get(), self.orientation)
    }
}

#[cfg(test)]
mod tests {}
