use crate::engine::func;
use crate::bitarr;
use crate::bitarray::BitArray;
use crate::middle_end::func::{PhysicalComponent, PhysicalInitContext, RelativeComponentBounds};

/// An input.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Input {
    sim: func::Input
}
impl Input {
    /// Creates a new instance of the input with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self { sim: func::Input::new(bitsize) }
    }
}
impl PhysicalComponent for Input {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) -> &'static str {
        "Input"
    }

    fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_from_bitsize(self.sim.get_bitsize())
    }
}

/// An output.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Output {
    sim: func::Output
}
impl Output {
    /// Creates a new instance of the output with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self { sim: func::Output::new(bitsize) }
    }
}
impl PhysicalComponent for Output {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) -> &'static str {
        "Output"
    }

    fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_from_bitsize(self.sim.get_bitsize())
    }
}

/// A constant.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Constant {
    sim: func::Constant
}
impl Constant {
    /// Creates a new instance of the constant with specified value.
    pub fn new(value: BitArray) -> Self {
        Self { sim: func::Constant::new(value) }
    }
}
impl PhysicalComponent for Constant {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) -> &'static str {
        "Constant"
    }

    fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_from_bitsize(self.sim.get_value().len())
    }
}

/// Power (essentially a constant 1).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Power;
impl Power {
    /// Creates a new instance of power.
    pub fn new() -> Self {
        Self
    }
}
impl PhysicalComponent for Power {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(func::Constant::new(bitarr![1]).into())
    }

    fn component_name(&self) -> &'static str {
        "Power"
    }

    fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_with_origin(2, 3, (1, 3))
    }
}

/// Ground (essentially a constant 0).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Ground;
impl Ground {
    /// Creates a new instance of ground.
    pub fn new() -> Self {
        Self
    }
}
impl PhysicalComponent for Ground {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(func::Constant::new(bitarr![0]).into())
    }

    fn component_name(&self) -> &'static str {
        "Ground"
    }

    fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_with_origin(2, 3, (1, 0))
    }
}

/// A splitter component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Splitter {
    sim: func::Splitter
}
impl Splitter {
    /// Creates a new instance of the splitter with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self { sim: func::Splitter::new(bitsize) }
    }
}
impl PhysicalComponent for Splitter {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) -> &'static str {
        "Splitter"
    }

    fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        let bitsize = i32::from(self.sim.get_bitsize());
        let mut ports = vec![(0, 0)];
        ports.extend((1..=bitsize).map(|i| (2 * i, 2)));

        RelativeComponentBounds::new((bitsize * 2, 2), ports)
    }
}

/// A tunnel.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Tunnel;
impl Tunnel {
    /// Creates a new instance of a tunnel.
    pub fn new() -> Self {
        Self
    }
}
impl PhysicalComponent for Tunnel {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        None
    }

    fn component_name(&self) ->  &'static str {
        "Tunnel"
    }

    fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_with_origin(3, 2, (3, 1))
    }
}

/// A probe.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Probe;
impl Probe {
    /// Creates a new instance of a probe.
    pub fn new() -> Self {
        Self
    }
}
impl PhysicalComponent for Probe {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        None
    }

    fn component_name(&self) ->  &'static str {
        "Probe"
    }

    fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port(2, 2)
    }
}

#[cfg(test)]
mod tests {}
